extern crate libc;
use libc::c_int;
use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, ForkResult};
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::io::FromRawFd;

// @note where does the user provide the prompt for the system call?
// https://github.com/Stichting-MINIX-Research-Foundation/minix/blob/master/minix/servers/pm/exec.c
mod system_calls {
    use super::*;
    // SYS_FORK
    pub fn sys_fork() {
        let pid = unsafe { fork() };
        match pid {
            Ok(ForkResult::Parent { child, .. }) => {
                println!("Child {} forked", child);
                waitpid(child, None).unwrap();
            }
            Ok(ForkResult::Child) => {
                println!("New child process");
                std::process::exit(1);
            }
            Err(_) => {
                // @todo EINVAL: Invalid parent process number or child slot to use.
                println!("Fork failed");
                std::process::exit(1);
            }
        }
        // unimplemented!();
    }

    // SYS_EXEC takes a command and executes it
    // @todo
    pub fn sys_exec() -> Result<(), nix::Error> {
        unimplemented!();

        // execvp(command_args[0], &command_args).expect("failed to execute command");
    }

    // SYS_EXIT
    pub fn sys_exit() {
        println!("Exiting");
        std::process::exit(1);
    }

    pub fn sys_wait() {
        //  Suspend the current process until one of its child processes terminates
    }

    // SYS_OPEN takes a file path and opens it for reading or writing or both
    pub fn sys_open(path: &str) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    // SYS_CLOSE closes a file descriptor
    // @note Rust automatically closes files when they go out of scope
    // but we can do it explicitly with this system call
    fn sys_close(fd: c_int) -> c_int {
        unsafe { libc::close(fd) }
    }
}
