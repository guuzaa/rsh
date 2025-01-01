use crate::error::Error;
use std::result;

pub type Builtin = fn(Vec<&str>) -> result::Result<(), Error>;

pub fn lsh_cd(args: Vec<&str>) -> result::Result<(), Error> {
    if args.len() != 2 {
        eprintln!("Usage: cd <directory>");
        return Err(Error::InvalidInput);
    }

    let path = args[1];
    std::env::set_current_dir(path).map_err(|_| Error::ChangeDirectoryError)
}

pub fn lsh_help(_: Vec<&str>) -> result::Result<(), Error> {
    println!("rsh: simple shell");
    println!("Type program names and arguments, and hit enter.");
    println!("The following are built in:");
    println!("cd\thelp\texit");
    Ok(())
}

pub fn lsh_exit(_: Vec<&str>) -> result::Result<(), Error> {
    std::process::exit(0);
}
