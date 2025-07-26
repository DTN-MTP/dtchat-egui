/// Macro for logging with file location information that makes logs clickable in VS Code
/// Automatically preserves context-based prefixes
#[macro_export]
macro_rules! log_with_location {
    // Pattern for println! with prefix
    ($($arg:tt)*) => {
        println!("[{}:{}]: {}", file!(), line!(), format!($($arg)*))
    };
}

/// Macro for error logging with file location information
#[macro_export]
macro_rules! elog_with_location {
    // Pattern for eprintln! with prefix
    ($($arg:tt)*) => {
        eprintln!("[{}:{}]: {}", file!(), line!(), format!($($arg)*))
    };

}
