use libc::c_char;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::process;
use tracing::{debug, error, info, span, Level};

mod builtins;
use crate::builtins::*;
mod error;
use crate::error::Error;

fn get_builtins() -> HashMap<&'static str, Builtin> {
    HashMap::from([
        ("cd", lsh_cd as Builtin),
        ("help", lsh_help as Builtin),
        ("exit", lsh_exit as Builtin),
    ])
}

fn rsh_loop() {
    loop {
        print!("rsh> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.split_whitespace().collect();
        debug!("Received command: {:?}", args);
        match rsh_execute(args) {
            Ok(_) => (),
            Err(e) => {
                error!("Command execution failed: {:?}", e);
                match e {
                    Error::InvalidInput => eprintln!("Invalid input"),
                    Error::ChangeDirectoryError => eprintln!("Failed to change directory"),
                }
            }
        }
    }
}

fn rsh_launch(args: Vec<&str>) {
    if args.is_empty() {
        return;
    }

    match unsafe { libc::fork() } {
        -1 => {
            eprintln!("Fork failed");
        }
        0 => {
            // Child process
            let program = std::ffi::CString::new(args[0]).unwrap();
            let args_cstring: Vec<std::ffi::CString> = args
                .iter()
                .map(|s| std::ffi::CString::new(*s).unwrap())
                .collect();
            let args_ptr: Vec<*const c_char> = args_cstring.iter().map(|s| s.as_ptr()).collect();
            let mut args_ptr_with_null = args_ptr.clone();
            args_ptr_with_null.push(std::ptr::null());

            unsafe {
                libc::execvp(program.as_ptr(), args_ptr_with_null.as_ptr());
                eprintln!("Command not found: {}", args[0]);
                process::exit(1);
            }
        }
        pid => {
            // Parent process
            let mut status = 0;
            loop {
                unsafe {
                    libc::waitpid(pid, &mut status, 0);
                }

                // wait until the child process has exited or been signaled
                if libc::WIFEXITED(status) || libc::WIFSIGNALED(status) {
                    break;
                }
            }
        }
    }
}

#[tracing::instrument]
fn rsh_execute(args: Vec<&str>) -> Result<(), Error> {
    let builtins = get_builtins();
    if args.is_empty() {
        debug!("Empty command received");
        return Ok(());
    }

    if builtins.contains_key(args[0]) {
        debug!(command = %args[0], "Executing builtin command");
        // debug!("Executing builtin command: {}", args[0]);
        builtins[args[0]](args)
    } else {
        debug!(command = %args[0], "Launching external command");
        // debug!("Launching external command: {}", args[0]);
        rsh_launch(args);
        Ok(())
    }
}

fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    info!("Starting rsh shell");
    rsh_loop();
    info!("Shell terminated");
    process::exit(0);
}
