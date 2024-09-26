// 'docker login' and 'docker logout' is not necessary, the calls just take a bollard::auth::DockerCredentials
pub mod container;
pub mod image;
pub mod network;
pub mod volume;

pub use super::{Error, Result};
use axum::body::Bytes;
use bollard::models::{ErrorDetail, ProgressDetail};
use futures_util::{Stream, StreamExt};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Debug, Eq, PartialEq)]
pub enum ByteStatus {
    Complete(usize),
    Partial(usize),
    Error(usize),
}

impl ByteStatus {
    fn complete(&mut self) {
        if let Self::Partial(data) = self {
            *self = Self::Complete(*data)
        }
    }

    fn add_bytes(&mut self, bytes: usize) {
        *self = match self {
            Self::Complete(current) => Self::Complete(*current + bytes),
            Self::Partial(current) => Self::Partial(*current + bytes),
            Self::Error(current) => Self::Error(*current + bytes),
        }
    }

    fn fail(&mut self) {
        *self = match self {
            Self::Complete(current) | Self::Partial(current) | Self::Error(current) => {
                Self::Error(*current)
            }
        }
    }
}

pub struct ByteResult<T> {
    pub status: Arc<Mutex<ByteStatus>>,
    pub handle: JoinHandle<Result<T>>,
}

async fn write_stream_to_file<T>(
    mut stream: T,
    path: &Path,
    status: Arc<Mutex<ByteStatus>>,
) -> Result<()>
where
    T: Stream<Item = Result<Bytes, bollard::errors::Error>> + Unpin,
{
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)
        .await?;
    while let Some(data) = stream.next().await {
        let data = data?;
        file.write_all(data.as_ref()).await?;
        status.lock().await.add_bytes(data.len());
    }
    status.lock().await.complete();
    Ok(())
}

async fn write_stream_to_memory<T>(mut stream: T, status: Arc<Mutex<ByteStatus>>) -> Result<Vec<u8>>
where
    T: Stream<Item = Result<Bytes, bollard::errors::Error>> + Unpin,
{
    let mut buffer = Vec::new();
    while let Some(data) = stream.next().await {
        let data = data?;
        buffer.write_all(data.as_ref()).await?;
        status.lock().await.add_bytes(data.len());
    }
    status.lock().await.complete();
    Ok(buffer)
}

#[derive(Debug, Default)]
pub struct Status {
    pub progress_map: BTreeMap<String, String>,
    pub finished: bool,
    pub errors: Vec<Error>,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Status: {}",
            if self.finished { "Finished" } else { "Ongoing" }
        )?;
        if !self.progress_map.is_empty() {
            for (id, status) in self.progress_map.iter() {
                writeln!(f, "{id}: {status}")?;
            }
        }
        if !self.errors.is_empty() {
            writeln!(f, "Errors:")?;
            for error in self.errors.iter() {
                writeln!(f, "{error}")?;
            }
        }
        Ok(())
    }
}

pub struct CallResult<T> {
    pub status: Arc<Mutex<Status>>,
    pub handle: JoinHandle<T>,
}

trait StatusUpdate {
    fn get_status_update(&self) -> Option<(String, String)>;
}

impl Status {
    fn add_update(&mut self, status_update: &dyn StatusUpdate) {
        if let Some((id, data)) = status_update.get_status_update() {
            self.progress_map.insert(id, data);
        }
    }

    fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    fn add_result<T>(&mut self, result: std::result::Result<T, Error>)
    where
        T: StatusUpdate,
    {
        match result {
            Ok(info) => self.add_update(&info),
            Err(error) => self.add_error(error),
        }
    }

    fn finish(&mut self) {
        self.finished = true;
    }
}

fn progress_detail_to_update(progress_detail: &Option<ProgressDetail>) -> String {
    match progress_detail {
        Some(ProgressDetail {
            current: Some(current),
            total: Some(total),
        }) => format!(" ({current}/{total})"),
        Some(ProgressDetail {
            current: Some(value),
            total: None,
        })
        | Some(ProgressDetail {
            current: None,
            total: Some(value),
        }) => format!(" ({value})"),
        _ => "".to_string(),
    }
}
fn error_detail_to_update(error_detail: &Option<ErrorDetail>) -> String {
    match error_detail {
        Some(ErrorDetail {
            code: Some(code),
            message: Some(message),
        }) => format!(": error code {code} ({message})"),
        Some(ErrorDetail {
            code: Some(code),
            message: None,
        }) => format!(": error code {code}"),
        Some(ErrorDetail {
            code: None,
            message: Some(message),
        }) => format!(": {message}"),
        _ => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    #[test]
    fn fail_byte_status_test() {
        let mut status = ByteStatus::Complete(1234);
        status.fail();
        assert_eq!(status, ByteStatus::Error(1234));
        let mut status = ByteStatus::Partial(6789);
        status.fail();
        assert_eq!(status, ByteStatus::Error(6789));
        let mut status = ByteStatus::Error(908);
        status.fail();
        assert_eq!(status, ByteStatus::Error(908));
    }

    #[test]
    fn add_byte_status_test() {
        let mut status = ByteStatus::Complete(1234);
        status.add_bytes(1234);
        assert_eq!(status, ByteStatus::Complete(2468));
        let mut status = ByteStatus::Partial(6789);
        status.add_bytes(11);
        assert_eq!(status, ByteStatus::Partial(6800));
        let mut status = ByteStatus::Error(908);
        status.add_bytes(2);
        assert_eq!(status, ByteStatus::Error(910));
    }

    #[test]
    fn complete_byte_status_test() {
        let mut status = ByteStatus::Complete(1234);
        status.complete();
        assert_eq!(status, ByteStatus::Complete(1234));
        let mut status = ByteStatus::Partial(6789);
        status.complete();
        assert_eq!(status, ByteStatus::Complete(6789));
        let mut status = ByteStatus::Error(908);
        status.complete();
        assert_eq!(status, ByteStatus::Error(908));
    }

    #[derive(Debug, Default)]
    struct TestStatusUpdate {
        empty: bool,
        id: String,
        text: String,
    }

    impl StatusUpdate for TestStatusUpdate {
        fn get_status_update(&self) -> Option<(String, String)> {
            if self.empty {
                None
            } else {
                Some((self.id.clone(), self.text.clone()))
            }
        }
    }

    #[test]
    fn update_status_test() {
        let mut status = Status::default();
        assert!(status.progress_map.is_empty());
        status.add_update(&TestStatusUpdate {
            empty: false,
            id: "TEST".to_string(),
            text: "TEXT".to_string(),
        });
        assert_eq!(
            status.progress_map,
            BTreeMap::from([("TEST".to_string(), "TEXT".to_string())])
        );
        status.add_update(&TestStatusUpdate {
            empty: false,
            id: "TSET".to_string(),
            text: "TXET".to_string(),
        });
        assert_eq!(
            status.progress_map,
            BTreeMap::from([
                ("TEST".to_string(), "TEXT".to_string()),
                ("TSET".to_string(), "TXET".to_string())
            ])
        );
        status.add_update(&TestStatusUpdate {
            empty: true,
            id: String::default(),
            text: String::default(),
        });
        assert_eq!(
            status.progress_map,
            BTreeMap::from([
                ("TEST".to_string(), "TEXT".to_string()),
                ("TSET".to_string(), "TXET".to_string())
            ])
        );
    }

    #[test]
    fn error_status_test() {
        let mut status = Status::default();
        assert!(status.errors.is_empty());
        status.add_error(anyhow!("Test error"));
        assert_eq!(status.errors.len(), 1);
        status.add_error(anyhow!("Test error 2"));
        assert_eq!(status.errors.len(), 2);
    }

    #[test]
    fn result_status_test() {
        let mut status = Status::default();
        assert!(status.errors.is_empty());
        status.add_result(Err::<TestStatusUpdate, _>(anyhow!("Test error")));
        assert_eq!(status.errors.len(), 1);
        status.add_result(Ok(TestStatusUpdate::default()));
        assert_eq!(status.errors.len(), 1);
        assert_eq!(
            status.progress_map,
            BTreeMap::from([("".to_string(), "".to_string()),])
        );
    }

    #[test]
    fn finish_status_test() {
        let mut status = Status::default();
        assert!(!status.finished);
        status.finish();
        assert!(status.finished);
    }

    #[test]
    fn progress_detail_to_update_test() {
        assert_eq!(progress_detail_to_update(&None), "");
        assert_eq!(
            progress_detail_to_update(&Some(ProgressDetail {
                total: Some(10),
                current: Some(5),
            })),
            " (5/10)"
        );
        assert_eq!(
            progress_detail_to_update(&Some(ProgressDetail {
                total: Some(70),
                current: None,
            })),
            " (70)"
        );
        assert_eq!(
            progress_detail_to_update(&Some(ProgressDetail {
                total: None,
                current: Some(22),
            })),
            " (22)"
        );
    }
    #[test]
    fn error_detail_to_update_test() {
        assert_eq!(error_detail_to_update(&None), "");
        assert_eq!(
            error_detail_to_update(&Some(ErrorDetail {
                code: Some(10),
                message: Some("1234".to_string()),
            })),
            ": error code 10 (1234)"
        );
        assert_eq!(
            error_detail_to_update(&Some(ErrorDetail {
                code: None,
                message: Some("asdfg".to_string()),
            })),
            ": asdfg"
        );
        assert_eq!(
            error_detail_to_update(&Some(ErrorDetail {
                code: Some(5768),
                message: None,
            })),
            ": error code 5768"
        );
    }
}
