use procfs::ProcError;

pub fn is_running(pid: i32) -> crate::Result<bool> {
    match procfs::process::Process::new(pid) {
        Ok(_) => Ok(true),
        Err(ProcError::NotFound(_)) => Ok(false),
        Err(e) => anyhow::bail!(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::timeout;

    fn sleepy_child() -> std::process::Child {
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
}
