use procfs::ProcError;

pub mod signal {
    pub use libc::{
        SIGABRT, SIGALRM, SIGBUS, SIGCHLD, SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, SIGIO, SIGKILL,
        SIGPIPE, SIGPOLL, SIGPROF, SIGPWR, SIGQUIT, SIGSEGV, SIGSTKFLT, SIGSTOP, SIGSYS, SIGTERM,
        SIGTSTP, SIGTTIN, SIGTTOU, SIGURG, SIGUSR1, SIGUSR2, SIGVTALRM, SIGWINCH, SIGXCPU, SIGXFSZ,
        SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK,
    };
}

pub fn is_running(pid: i32) -> crate::Result<bool> {
    match procfs::process::Process::new(pid) {
        Ok(_) => Ok(true),
        Err(ProcError::NotFound(_)) => Ok(false),
        Err(e) => anyhow::bail!(e),
    }
}

pub fn send_signal(pid: i32, signal: libc::c_int) -> crate::Result<()> {
    let result = unsafe { libc::kill(pid, signal) };
    if result == 0 {
        return Ok(());
    }

    match std::io::Error::last_os_error().raw_os_error().unwrap_or(0) {
        0 => Ok(()),
        libc::EINVAL => Err(anyhow::anyhow!("Invalid signal {signal}")),
        libc::EPERM => Err(anyhow::anyhow!("No permission to send the signal {signal}")),
        libc::ESRCH => Err(anyhow::anyhow!(
            "Specified process(group) {pid} does not exist or is zombie"
        )),
        x => Err(anyhow::anyhow!(
            "Unexpected error sending signal {signal} to pid {pid}: {}",
            std::io::Error::from_raw_os_error(x)
        )),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ntest::timeout;

    pub fn sleepy_child() -> std::process::Child {
        std::process::Command::new("sleep")
            .arg("100d")
            .spawn()
            .unwrap()
    }

    #[test]
    #[timeout(10000)]
    fn is_running_true() {
        let mut child = sleepy_child();
        let is_running_result = is_running(child.id() as i32);
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(is_running_result.unwrap());
    }

    #[test]
    #[timeout(10000)]
    fn is_running_false() {
        let mut child = sleepy_child();
        let child_id = child.id() as i32;
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(!is_running(child_id).unwrap());
    }

    #[test]
    #[timeout(10000)]
    fn send_signal_ok() {
        let mut child = sleepy_child();
        let child_id = child.id() as i32;
        match send_signal(child_id, signal::SIGKILL) {
            Ok(()) => {
                child.wait().unwrap();
            }
            Err(e) => {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("Signal could not be sent: {e}")
            }
        }
    }

    #[test]
    #[timeout(10000)]
    fn send_signal_err() {
        let mut child = sleepy_child();
        let child_id = child.id() as i32;
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(send_signal(child_id, 0).is_err());
    }

    #[test]
    #[timeout(10000)]
    fn send_signal_invalid() {
        let mut child = sleepy_child();
        let result = send_signal(child.id() as i32, 100);
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(result.is_err());
    }
}
