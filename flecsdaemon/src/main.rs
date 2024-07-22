const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const CODENAME: Option<&str> = option_env!("CODENAME");
fn main() {
    println!("Running flecsd version {}-{}", VERSION.unwrap_or("unknown"), CODENAME.unwrap_or("unknown"));
}
