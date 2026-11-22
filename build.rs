fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let icon = std::path::Path::new("assets/app.ico");
        if icon.exists() {
            let _ = winresource::WindowsResource::new().set_icon("assets/app.ico").compile();
        }
    }
}
