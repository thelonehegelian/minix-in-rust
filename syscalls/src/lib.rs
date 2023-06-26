use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, ForkResult};

// @note where does the user provide the prompt for the system call?
mod system_calls {
    // SYS_FORK
    pub fn sys_fork() -> ForkResult {
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
    }

    pub fn sys_exec() -> Result {

        // execvp(command_args[0], &command_args).expect("failed to execute command");
    }
}
