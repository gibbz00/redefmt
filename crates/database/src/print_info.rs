pub struct PrintInfo {
    /// None for regular `println!()` statements
    level: Option<LogLevel>,
    location: Location,
}

// File / module inferred by DB file path?
// Remaining would be line
pub struct Location;

pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
