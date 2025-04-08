use crate::sorcerer::exportius::Exportius;
use flecsd_axum_server::apis::flecsport::{
    ExportsExportIdDeleteResponse as DeleteResponse, ExportsExportIdGetResponse as GetResponse,
};
use flecsd_axum_server::models;
use flecsd_axum_server::models::{
    ExportsExportIdDeletePathParams as DeletePathParams,
    ExportsExportIdGetPathParams as GetPathParams,
};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn get<E: Exportius>(exportius: Arc<E>, path_params: GetPathParams) -> GetResponse {
    let export_dir = PathBuf::from(crate::lore::flecsport::BASE_PATH);
    match exportius
        .get_export(&export_dir, path_params.export_id)
        .await
    {
        Ok(Some(path)) => GetResponse::Status200_Success(path),
        Ok(None) => GetResponse::Status404_ExportNotFound,
        Err(e) => {
            GetResponse::Status500_InternalServerError(models::AdditionalInfo::new(e.to_string()))
        }
    }
}

pub async fn delete<E: Exportius>(
    exportius: Arc<E>,
    path_params: DeletePathParams,
) -> DeleteResponse {
    let export_dir = PathBuf::from(crate::lore::flecsport::BASE_PATH);
    match exportius
        .delete_export(&export_dir, path_params.export_id)
        .await
    {
        Ok(true) => DeleteResponse::Status200_Success,
        Ok(false) => DeleteResponse::Status404_ExportNotFound,
        Err(e) => DeleteResponse::Status500_InternalServerError(models::AdditionalInfo::new(
            e.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorcerer::exportius::MockExportius;
    use mockall::predicate;
    use testdir::testdir;

    #[tokio::test]
    async fn get_200() {
        let path = testdir!().join("file");
        let expected_path = path.clone();
        let mut exportius = MockExportius::new();
        exportius
            .expect_get_export()
            .once()
            .with(predicate::always(), predicate::eq("12345".to_string()))
            .return_once(move |_, _| Ok(Some(path)));
        assert_eq!(
            get(
                Arc::new(exportius),
                GetPathParams {
                    export_id: "12345".to_string()
                }
            )
            .await,
            GetResponse::Status200_Success(expected_path)
        );
    }

    #[tokio::test]
    async fn get_404() {
        let mut exportius = MockExportius::new();
        exportius
            .expect_get_export()
            .once()
            .with(predicate::always(), predicate::eq("12345".to_string()))
            .returning(|_, _| Ok(None));
        assert_eq!(
            get(
                Arc::new(exportius),
                GetPathParams {
                    export_id: "12345".to_string()
                }
            )
            .await,
            GetResponse::Status404_ExportNotFound,
        );
    }

    #[tokio::test]
    async fn get_500_open() {
        let mut exportius = MockExportius::new();
        exportius
            .expect_get_export()
            .once()
            .with(predicate::always(), predicate::eq("12345".to_string()))
            .returning(|_, _| Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)));
        assert!(matches!(
            get(
                Arc::new(exportius),
                GetPathParams {
                    export_id: "12345".to_string()
                }
            )
            .await,
            GetResponse::Status500_InternalServerError(_),
        ));
    }

    #[tokio::test]
    async fn delete_200() {
        let mut exportius = MockExportius::new();
        exportius
            .expect_delete_export()
            .once()
            .with(predicate::always(), predicate::eq("12345".to_string()))
            .returning(|_, _| Ok(true));
        assert!(matches!(
            delete(
                Arc::new(exportius),
                DeletePathParams {
                    export_id: "12345".to_string()
                }
            )
            .await,
            DeleteResponse::Status200_Success,
        ));
    }

    #[tokio::test]
    async fn delete_404() {
        let mut exportius = MockExportius::new();
        exportius
            .expect_delete_export()
            .once()
            .with(predicate::always(), predicate::eq("12345".to_string()))
            .returning(|_, _| Ok(false));
        assert!(matches!(
            delete(
                Arc::new(exportius),
                DeletePathParams {
                    export_id: "12345".to_string()
                }
            )
            .await,
            DeleteResponse::Status404_ExportNotFound,
        ));
    }

    #[tokio::test]
    async fn delete_500() {
        let mut exportius = MockExportius::new();
        exportius
            .expect_delete_export()
            .once()
            .with(predicate::always(), predicate::eq("12345".to_string()))
            .returning(|_, _| Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)));
        assert!(matches!(
            delete(
                Arc::new(exportius),
                DeletePathParams {
                    export_id: "12345".to_string()
                }
            )
            .await,
            DeleteResponse::Status500_InternalServerError(_),
        ));
    }
}
