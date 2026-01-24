pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(debug_assertions)]        pub const BUILD_TYPE: &str = "debug";
#[cfg(not(debug_assertions))]   pub const BUILD_TYPE: &str = "release";
#[cfg(debug_assertions)]        pub const BUILD_IS_DEBUG: bool = true;
#[cfg(not(debug_assertions))]   pub const BUILD_IS_DEBUG: bool = false;
pub const BUILD_ID: &str = env!("BUILD_ID");