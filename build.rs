#[cfg(windows)]
fn main() {
    if std::path::Path::new("resources/app.rc").exists() {
        embed_resource::compile("resources/app.rc", embed_resource::NONE);
    }
}

#[cfg(not(windows))]
fn main() {}

