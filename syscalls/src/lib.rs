/**  @note path as &str works for some functions where path is expected to be a pointer in the syscall
*    but not for others, so we use CString and convert it to a raw pointer with as_ptr()
*    https://doc.rust-lang.org/std/ffi/struct.CString.html
*    @todo we need to make path handling consistent
*    @note perhaps its best to use types from libc for the system calls for consistency and clarity
*    @todo replace isize return types with Result<isize, nix::Error> ? or just Result?
*    @todo think about what sort of error loggin we need to do for the system calls, simlply generating an error is not enough
*/
extern crate libc;
use libc::off_t;
use libc::{c_int, c_long, creat, fcntl, lseek, EINVAL, F_GETFD};
use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, ForkResult};
use std::any::TypeId;
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::RawFd;

// @note where does the user provide the prompt for the system call?
// https://github.com/Stichting-MINIX-Research-Foundation/minix/blob/master/minix/servers/pm/exec.c
mod system_calls {
    use libc::time_t;

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

    // SYS_READ reads from a file descriptor
    // @note low level system call
    // The read() system call is used to read data from a file descriptor into a buffer.
    pub fn sys_read(fd: c_int, buf: &mut [u8]) -> io::Result<usize> {
        let mut file = unsafe { File::from_raw_fd(fd) };
        // error handling, the descriptor may not be valid
        let bytes_read = match file.read(buf) {
            Ok(bytes) => bytes,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => 0,
            Err(e) => return Err(e),
        };
        Ok(bytes_read)
    }

    // SYS_LSEEK repositions the offset of the open file associated with the file descriptor
    pub fn sys_lseek(fd: RawFd, offset: isize, whence: c_int) -> std::io::Result<isize> {
        // lets make sure the file descriptor is valid and the offset is valid
        unsafe {
            if fcntl(fd, F_GETFD) == -1 {
                return Err(std::io::Error::last_os_error());
            }
        }
        let ret = unsafe { lseek(fd, offset as off_t, whence) };
        if ret == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(ret as isize)
        }
    }

    pub fn sys_create(path: &str, permissions: u16) -> std::io::Result<c_int> {
        let c_path = std::ffi::CString::new(path)?;
        let fd = unsafe { creat(path.as_ptr() as *const i8, permissions as libc::mode_t) };
        if fd == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(fd)
        }
    }

    // SYS_UNLINK deletes a name from the filesystem
    pub fn sys_unlink(path: &str) -> std::io::Result<isize> {
        let c_str = std::ffi::CString::new(path)?;
        let ret = unsafe { libc::unlink(path.as_ptr() as *const i8) };
        if ret == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(ret as isize)
        }
    }

    pub fn sys_chmod(path: &str, permissions: u16) -> std::io::Result<isize> {
        let c_path = std::ffi::CString::new(path)?;
        let ret = unsafe { libc::chmod(c_path.as_ptr() as *const i8, permissions as libc::mode_t) };
        if ret == -1 {
            return Err(std::io::Error::last_os_error());
        }
        // set permissions for the file
        let metadata = std::fs::metadata(path)?;
        let mut file_permissions = metadata.permissions();
        let mode = file_permissions.mode();
        // @note I know jack about biwise operations!
        let new_mode = (mode & !0o7777) | (permissions as u32 & 0o7777);
        file_permissions.set_mode(new_mode);

        std::fs::set_permissions(path, file_permissions)?;
        Ok(ret as isize)
    }

    pub fn sys_chown(
        path: &str,
        new_user_id: usize,
        new_group_id: usize,
    ) -> std::io::Result<usize> {
        let c_path = std::ffi::CString::new(path)?;
        let ret = unsafe {
            libc::chown(
                c_path.as_ptr() as *const i8,
                new_user_id as libc::uid_t,
                new_group_id as libc::gid_t,
            )
        };
        if ret == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(ret as usize)
    }

    pub fn sys_chdir(path: &str) -> std::io::Result<()> {
        let c_path = std::ffi::CString::new(path)?;
        let ret = unsafe { libc::chdir(c_path.as_ptr() as *const i8) };
        if ret == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn sys_getpid() -> std::io::Result<(usize)> {
        let pid = unsafe { libc::getpid() };
        if pid == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(pid as usize)
    }

    pub fn sys_getppid() -> std::io::Result<(usize)> {
        let ppid = unsafe { libc::getppid() };
        if ppid == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(ppid as usize)
    }

    pub fn sys_kill(pid: isize, signal_num: isize) -> std::io::Result<usize> {
        let ret = unsafe { libc::kill(pid as i32, signal_num as i32) };
        if ret == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(ret as usize)
    }

    // * 1. Call the sbrk system call to change the heap size
    // * 2. Check if the sbrk system call was successful
    // * 3. Return the updated heap address if the sbrk call was successful
    // * 4. Return an error if the sbrk call was not successful
    pub fn sys_sbrk(increment: libc::intptr_t) -> std::io::Result<libc::intptr_t> {
        unsafe {
            let curr_brk = libc::sbrk(0);
            if curr_brk == libc::sbrk(-1) {
                return Err(std::io::Error::last_os_error());
            }
            let new_brk = libc::sbrk(increment as i32);
            if new_brk == libc::sbrk(-1) {
                return Err(std::io::Error::last_os_error());
            }
            Ok(new_brk as libc::intptr_t)
        }
    }

    pub fn sys_time() -> std::io::Result<libc::time_t> {
        let time = unsafe { libc::time(std::ptr::null_mut()) };
        if time == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(time)
    }

    pub fn sys_sleep(seconds: usize) -> io::Result<libc::c_uint> {
        let ret = unsafe { libc::sleep(seconds as libc::c_uint) };
        if ret != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(ret)
    }
}
