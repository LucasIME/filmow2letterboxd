pub fn setup_logging() {
    let mut builder = env_logger::Builder::new();

    builder.filter(None, log::LevelFilter::Info);

    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    }

    builder.init();
}
