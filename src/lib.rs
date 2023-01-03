#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
    Off,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Off => write!(f, "OFF"),
        }
    }
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! log {
    ($message:expr, $level:expr) => {
        // get the current local date and time
        let now = chrono::Local::now();
        let datetime = now.format("%Y-%m-%d %H:%M:%S%.3f %Z");

        // get the absolute file path
        let path = std::path::Path::new(file!()).display();
        let line = line!();
        let thread = std::thread::current().name().unwrap_or("main").to_string();
        // let fn_name = fn_name!();

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("debug.log")
            .unwrap();

        if let Err(e) = writeln!(
            file,
            "[{}] [{}] [{}] [{}:{}] {}",
            datetime, $level, thread, path, line, $message,
        ) {
            eprintln!("Couldn't write to file: {}", e);
        }
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! info {
    ($message:expr) => {
        $crate::log!($message, $crate::LogLevel::Info);
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! debug {
    ($message:expr) => {
        $crate::debug::log::log!($message, $crate::debug::LogLevel::Debug);
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! error {
    ($message:expr) => {
        $crate::log!($message, $crate::LogLevel::Error);
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! warning {
    ($message:expr) => {
        $crate::debug::log::log!($message, $crate::debug::LogLevel::Warning);
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! trace {
    ($message:expr) => {
        $crate::debug::log::log!($message, $crate::debug::LogLevel::Trace);
    };
}
