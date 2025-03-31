extern crate proc_macro;

#[macro_export]
macro_rules! env_parse {
    ($env_var:expr, $default:expr, $t:ty) => {
        std::env::var($env_var)
            .ok()
            .and_then(|s| s.parse::<$t>().ok())
            .unwrap_or($default)
    };
}
