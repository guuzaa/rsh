use libc::c_char;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::process;

mod builtins;
use crate::builtins::*;
mod error;
use crate::error::Error;

fn get_builtins() -> HashMap<&'static str, fn(Vec<&str>) -> Result<(), Error>> {
    HashMap::from([
        ("cd", lsh_cd as fn(Vec<&str>) -> Result<(), Error>),
        ("help", lsh_help as fn(Vec<&str>) -> Result<(), Error>),
        ("exit", lsh_exit as fn(Vec<&str>) -> Result<(), Error>),
    ])
}

fn rsh_loop() {
    loop {
        print!("rsh> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.split_whitespace().collect();
        match rsh_execute(args) {
            Ok(_) => (),
            Err(e) => match e {
                Error::InvalidInput => eprintln!("Invalid input"),
                Error::ChangeDirectoryError => eprintln!("Failed to change directory"),
            },
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
            return;
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

fn rsh_execute(args: Vec<&str>) -> Result<(), Error> {
    let builtins = get_builtins();
    if builtins.contains_key(args[0]) {
        builtins[args[0]](args)
    } else {
        rsh_launch(args);
        Ok(())
    }
}

fn main() {
    rsh_loop();
    process::exit(0);
}
