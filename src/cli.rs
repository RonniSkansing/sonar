pub const APP_NAME: &str = "sonar";
pub const APP_AUTHOR: &str = "Ronni Skansing";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_ABOUT: &str = "portable monitoring";

pub const DEBUG_ARG_NAME: &str = "debug";
pub const DEBUG_ARG_SHORT: &str = "d";
pub const DEBUG_ARG_LONG: &str = "debug";
pub const DEBUG_ARG_TAKES_VALUE: bool = false;
pub const DEBUG_ARG_HELP: &str = "Add a backtrace (if build with symbols)";

pub const QUIET_ARG_NAME: &str = "quiet";
pub const QUIET_ARG_SHORT: &str = "q";
pub const QUIET_ARG_LONG: &str = "quiet";
pub const QUIET_ARG_TAKES_VALUE: bool = false;
pub const QUIET_ARG_HELP: &str = "No output";
