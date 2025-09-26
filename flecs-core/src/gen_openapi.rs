use flecs_core::fsm::ApiDoc;
use utoipa::OpenApi;
fn main() {
    let api = ApiDoc::openapi();
    println!("{api:#?}");
    std::fs::write("./api-spec.yaml", api.to_yaml().unwrap()).unwrap();
}
