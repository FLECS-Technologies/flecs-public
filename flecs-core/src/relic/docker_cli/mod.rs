use crate::jeweler::app::Token;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tracing::trace;

#[derive(thiserror::Error, Debug)]
pub enum ExecuteCommandError {
    #[error("IO error during execution of command: {0}")]
    IO(#[from] std::io::Error),
    #[error("Command returned code {exit_status}: {stderr}")]
    CommandFailed {
        exit_status: ExitStatus,
        stderr: String,
    },
}

pub struct DockerCli {
    socket_path: PathBuf,
}

impl DockerCli {
    pub fn new_with_unix_socket(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    fn command(&self) -> Command {
        let mut command = Command::new("docker");
        command
            .arg("--host")
            .arg(format!("unix://{}", self.socket_path.to_string_lossy()));
        command
    }

    async fn spawn_printing_stdout<T: AsRef<[u8]>>(
        mut command: Command,
        stdin: Option<&T>,
    ) -> Result<(), ExecuteCommandError> {
        let mut child = command
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        if let (Some(mut stdin_pipe), Some(stdinput)) = (child.stdin.take(), stdin) {
            stdin_pipe.write_all(stdinput.as_ref()).await?;
        }
        let stdout = child.stdout.take().expect("");
        let reader = BufReader::new(stdout);
        let mut line_reader = reader.lines();
        while let Some(line) = line_reader.next_line().await? {
            trace!("{line}");
        }
        let output = child.wait_with_output().await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(ExecuteCommandError::CommandFailed {
                exit_status: output.status,
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    async fn spawn_with_stdout<T: AsRef<[u8]>>(
        mut command: Command,
        stdin: Option<&T>,
    ) -> Result<Vec<u8>, ExecuteCommandError> {
        let mut child = command
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        if let (Some(mut stdin_pipe), Some(stdinput)) = (child.stdin.take(), stdin) {
            stdin_pipe.write_all(stdinput.as_ref()).await?;
        }
        let output = child.wait_with_output().await?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(ExecuteCommandError::CommandFailed {
                exit_status: output.status,
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    pub async fn login(&self, token: Token) -> Result<(), ExecuteCommandError> {
        let mut command = self.command();
        command.args([
            "login",
            "--username",
            token.username.as_str(),
            "--password-stdin",
            "flecs.azurecr.io",
        ]);
        Self::spawn_printing_stdout(command, Some(&token.password)).await
    }

    pub async fn logout(&self) -> Result<(), ExecuteCommandError> {
        let mut command = self.command();
        command.arg("logout");
        Self::spawn_printing_stdout::<&str>(command, None).await
    }

    pub async fn compose_pull<T: AsRef<[u8]>>(
        &self,
        project_name: &str,
        compose: &T,
    ) -> Result<(), ExecuteCommandError> {
        let mut command = self.command();
        command.args([
            "compose",
            "--project-name",
            project_name,
            "--file",
            "-",
            "pull",
        ]);
        Self::spawn_printing_stdout(command, Some(compose)).await
    }

    pub async fn compose_up<T: AsRef<[u8]>>(
        &self,
        project_name: &str,
        workdir: &Path,
        compose: &T,
    ) -> Result<(), ExecuteCommandError> {
        let mut command = self.command();
        command.args([
            "compose",
            "--project-name",
            project_name,
            "--project-directory",
            workdir.to_string_lossy().as_ref(),
            "--file",
            "-",
            "up",
            "--detach",
        ]);
        Self::spawn_printing_stdout(command, Some(compose)).await
    }

    pub async fn compose_stop<T: AsRef<[u8]>>(
        &self,
        project_name: &str,
        compose: &T,
    ) -> Result<(), ExecuteCommandError> {
        let mut command = self.command();
        command.args([
            "compose",
            "--project-name",
            project_name,
            "--file",
            "-",
            "stop",
        ]);
        Self::spawn_printing_stdout(command, Some(compose)).await
    }

    pub async fn compose_remove<T: AsRef<[u8]>>(
        &self,
        project_name: &str,
        compose: &T,
    ) -> Result<(), ExecuteCommandError> {
        let mut command = self.command();
        command.args([
            "compose",
            "--project-name",
            project_name,
            "--file",
            "-",
            "rm",
            "--force",
        ]);
        Self::spawn_printing_stdout(command, Some(compose)).await
    }

    pub async fn compose_containers<T: AsRef<[u8]>>(
        &self,
        project_name: &str,
        compose: &T,
    ) -> Result<Vec<String>, ExecuteCommandError> {
        let mut command = self.command();
        command.args([
            "compose",
            "--project-name",
            project_name,
            "--file",
            "-",
            "ps",
            "--quiet",
            "--all",
        ]);
        let stdout = Self::spawn_with_stdout(command, Some(compose)).await?;
        let stdout = String::from_utf8_lossy(&stdout);
        Ok(stdout.split_whitespace().map(str::to_string).collect())
    }

    pub async fn compose_logs<T: AsRef<[u8]>>(
        &self,
        project_name: &str,
        compose: &T,
    ) -> Result<String, ExecuteCommandError> {
        let mut command = self.command();
        command.args([
            "compose",
            "--project-name",
            project_name,
            "--file",
            "-",
            "logs",
        ]);
        let stdout = Self::spawn_with_stdout(command, Some(compose)).await?;
        let stdout = String::from_utf8_lossy(&stdout);
        Ok(stdout.to_string())
    }
}
