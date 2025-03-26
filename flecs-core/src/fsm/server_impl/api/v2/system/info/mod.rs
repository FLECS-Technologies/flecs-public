use flecsd_axum_server::apis::system::SystemInfoGetResponse as GetResponse;
use tracing::error;

pub fn get() -> Result<GetResponse, ()> {
    Ok(GetResponse::Status200_Sucess(
        crate::relic::system::info::try_create_system_info().map_err(|e| {
            error!("Could not create SystemInfo: {e}");
        })?,
    ))
}
