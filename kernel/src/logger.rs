
/* enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Panic,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Panic => "KERNEL PANIC",
        };

        f.write_str(s);

        Ok(())
    }
}

// #[derive(Debug)]
struct Log {
    file: &'static str,
    line: usize,
    level: LogLevel,
    msg: String,
} */
