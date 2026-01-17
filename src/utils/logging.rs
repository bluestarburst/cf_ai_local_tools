use std::env;

pub fn init() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    // ignore error if logger already set
    let _ = env_logger::try_init();
}
