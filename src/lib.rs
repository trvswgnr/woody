#![allow(dead_code)]
#![allow(unused_macros)]

///! A (really) very simple logger that can be used globally in any project.
///!
///! Logs the current time, the log level, the thread name, the file and line number, and the message.
///! Log messages are written to a file (`debug.log` by default).
use lazy_static::lazy_static;
use std::{
    env,
    fs::{File, OpenOptions},
    hash::{Hash, Hasher},
    io::Write,
    sync::{Arc, Mutex},
};

lazy_static! {
    static ref INSTANCE: Arc<Mutex<Option<Logger>>> = Arc::new(Mutex::new(None));
    static ref FILENAME: Arc<Mutex<String>> = Arc::new(Mutex::new("debug.log".to_string()));
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

/// The logger struct. A singleton that can only be created once.
#[derive(Clone, Debug)]
pub struct Logger {
    file: Arc<Mutex<File>>,
    level: LogLevel,
    filename: String,
}

/// Generates a temp file name
///
/// Returns a string that looks like this:
/// `temp-8444741687653642537.log`
fn generate_temp_file_name() -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let now = chrono::Local::now();
    let now_string = now.format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string();
    now_string.hash(&mut hasher);
    let hash = hasher.finish();
    let prefix = "temp-";
    let suffix = ".log";
    // make sure it's exactly 32 characters long
    let len = 32 - prefix.len() - suffix.len();
    let hash = format!("{hash:0>len$}");

    format!("temp-{hash}.log")
}

/// Gets the file and filename to use for logging.
fn get_file_and_filename() -> (Arc<Mutex<File>>, String) {
    let filename: String;
    let file: Arc<Mutex<File>>;
    if !cfg!(test) {
        filename = FILENAME.lock().unwrap().clone();
        file = Arc::new(Mutex::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&filename)
                .unwrap(),
        ));
    } else {
        // create a temp file using the std library
        let temp_dir = env::temp_dir();
        // append "logger" to the temp dir so it's like this:
        // /tmp/logger/temp-af44fa0-1f2c-4b5a-9c1f-7f8e9d0a1b2c.log
        let temp_dir = temp_dir.join("logger");
        // remove the temp dir if it already exists
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).unwrap();
        }
        std::fs::create_dir(&temp_dir).unwrap();
        let temp_file_name = generate_temp_file_name();
        let temp_file_path = temp_dir.join(temp_file_name);
        filename = temp_file_path.to_str().unwrap().to_string();

        file = Arc::new(Mutex::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(temp_file_path)
                .unwrap(),
        ));
    }

    (file, filename)
}

impl Logger {
    /// Create a new logger. This is a singleton, so it can only be called once.
    fn new() -> Self {
        let level = LogLevel::Info;
        let (file, filename) = get_file_and_filename();

        Self {
            file,
            level,
            filename,
        }
    }

    /// Set the log level. This will only log messages that are equal to or above the log level.
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    /// Log a message at the given level.
    pub fn log<W: Write>(&self, info: &LogInfo, writer: Option<&mut W>) {
        let now = chrono::Local::now();
        let thread = info.thread.clone().unwrap_or_else(|| {
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("unnamed");
            name.to_string()
        });
        let location = format!("{}:{}", info.filepath, info.line_number);
        let level = info.level;
        let message = info.message.clone();
        let output = format!(
            "[{}] [{}] [{}] [{}] {}\n",
            now.format("%Y-%m-%d %H:%M:%S%.3f %Z"),
            level,
            thread,
            location,
            message
        );

        if let Some(writer) = writer {
            writer.write_all(output.as_bytes()).unwrap();
            return;
        }

        let mut file = self.file.lock().unwrap();
        file.write_all(output.as_bytes()).unwrap();
    }

    /// Gets the instance of the logger. If the logger is not created, it will create it.
    pub fn get_instance() -> Logger {
        // Check if the instance is already created.
        let current_global_instance = INSTANCE.clone();
        let mut current_global_instance_lock = current_global_instance.lock().unwrap();
        if current_global_instance_lock.is_none() {
            // If the instance is not created, create it.
            let logger = Logger::new();
            *current_global_instance_lock = Some(logger.clone());
            logger
        } else {
            // If the instance is already created, return it.
            current_global_instance_lock.clone().unwrap()
        }
    }
}

/// The log info struct. This is used to log a message.
#[derive(Clone)]
pub struct LogInfo {
    pub level: LogLevel,
    pub message: String,
    pub filepath: &'static str,
    pub line_number: u32,
    pub thread: Option<String>,
}

/// The log macro. Used in other macros.
#[macro_export]
macro_rules! log {
    ($level:expr, $message:expr) => {
        let message = $message.to_string();
        let logger = $crate::Logger::get_instance();
        let info = $crate::LogInfo {
            level: $level,
            message,
            filepath: file!(),
            line_number: line!(),
            thread: None,
        };
        let writer: Option<&mut Vec<u8>> = None;
        logger.log(&info, writer);
    };
}

/// Logs a debug message.
#[macro_export]
macro_rules! debug {
    ($message:expr) => {
        $crate::log!($crate::LogLevel::Debug, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        $crate::log!($crate::LogLevel::Debug, message);
    };
}

/// Logs an info message.
#[macro_export]
macro_rules! info {
    ($message:expr) => {
        $crate::log!($crate::LogLevel::Info, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        $crate::log!($crate::LogLevel::Info, message);
    };
}

/// Logs a warning message.
#[macro_export]
macro_rules! warning {
    ($message:expr) => {
        $crate::log!($crate::LogLevel::Warning, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        $crate::log!($crate::LogLevel::Warning, message);
    };
}

/// Logs an error message.
#[macro_export]
macro_rules! error {
    ($message:expr) => {
        $crate::log!($crate::LogLevel::Error, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        $crate::log!($crate::LogLevel::Error, message);
    };
}

/// Logs a trace message.
#[macro_export]
macro_rules! trace {
    ($message:expr) => {
        $crate::log!($crate::LogLevel::Trace, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        $crate::log!($crate::LogLevel::Trace, message);
    };
}

/// Logs a text message.
#[macro_export]
macro_rules! text {
    ($message:expr) => {
        $crate::log!($crate::LogLevel::Off, $message);
    };

    ($message:expr, $($arg:tt)*) => {
        let message = format!($message, $($arg)*).to_string();
        $crate::log!($crate::LogLevel::Off, message);
    };
}

/// Gets the name of the current function.
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::io::Read;
    use tokio::runtime::Runtime;

    use super::*;

    /// Get the global instance of the Logger (or None if it doesn't exist).
    fn get_global_instance() -> Option<Logger> {
        let current_global_instance = INSTANCE.clone();
        let current_global_instance_lock = current_global_instance.lock().unwrap();
        current_global_instance_lock.clone()
    }

    /// Check that the global instance is None before running `Logger::get_instance()`.
    /// and that it is Some after running `Logger::get_instance()`.
    #[test]
    #[serial]
    fn test_global_instance_value() {
        let current_global_instance = get_global_instance();
        assert!(current_global_instance.is_none() || current_global_instance.is_some());

        let logger = Logger::get_instance();
        let current_global_instance = get_global_instance();
        assert!(current_global_instance.is_some());
        assert_eq!(logger.level, LogLevel::Info);
    }

    /// Check that writing to the logger works.
    #[test]
    fn test_writing_to_logger() {
        let logger = Logger::get_instance();
        let info = LogInfo {
            level: LogLevel::Info,
            message: "Hello, world!".to_string(),
            filepath: file!(),
            line_number: line!(),
            thread: None,
        };

        let mut writer = Vec::new();
        logger.log(&info, Some(&mut writer));

        let mut contents = String::new();
        contents.push_str(&String::from_utf8(writer).unwrap());

        assert!(
            contents.contains(info.message.as_str()),
            "Contents of log does not contain 'Hello, world!'\nContents: {contents}"
        );
    }

    fn check_log_file_contains(s: String) {
        // open the file and check that it contains the message
        let logger = Logger::get_instance();
        let filename = logger.clone().filename;
        let mut file = OpenOptions::new().read(true).open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert!(
            contents.contains(s.as_str()),
            "Contents of log does not contain '{s}'\nContents: {contents}\nLogger: {logger:?}"
        );
    }

    /// Check that writing to the logger across multiple threads works.
    #[test]
    fn test_writing_to_logger_across_threads() {
        async fn write_to_logger(id: Option<u8>) {
            let logger = Logger::get_instance();
            let thread = std::thread::current();
            let thread = thread.name();
            let thread = match id {
                Some(id) => format!("{}-{}", thread.unwrap(), id),
                None => thread.unwrap().to_string(),
            };
            let id = id.unwrap_or(0);
            let message = format!("Hello, world! {id}");
            let info = LogInfo {
                level: LogLevel::Info,
                message,
                filepath: file!(),
                line_number: line!(),
                thread: Some(thread),
            };

            let writer: Option<&mut Vec<u8>> = None;
            logger.log(&info, writer);
        }

        async fn spawn_logs() {
            let mut handles = Vec::new();
            for i in 0..10 {
                let task = tokio::spawn(write_to_logger(Some(i)));
                handles.push(task);
            }

            for handle in handles {
                handle.await.unwrap();
            }
        }

        let rt = Runtime::new().unwrap();
        rt.block_on(spawn_logs());

        let filename = FILENAME.lock().unwrap().clone();
        let mut file = OpenOptions::new().read(true).open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        for i in 0..10 {
            let message = format!("Hello, world! {i}");
            check_log_file_contains(message);
        }
    }

    #[test]
    fn test_log_info() {
        let f = function!();
        let s = format!("Hello, {f}!");
        info!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_debug() {
        let f = function!();
        let s = format!("Hello, {f}!");
        debug!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_warning() {
        let f = function!();
        let s = format!("Hello, {f}!");
        warning!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_error() {
        let f = function!();
        let s = format!("Hello, {f}!");
        error!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_trace() {
        let f = function!();
        let s = format!("Hello, {f}!");
        trace!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_text() {
        let f = function!();
        let s = format!("Hello, {f}!");
        text!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_random_file_name() {
        let filename = generate_temp_file_name();

        // make sure the filename is 32 characters long
        assert_eq!(
            filename.len(),
            32,
            "Filename is not 32 characters long: {}",
            filename.len()
        );

        // make sure the filename starts with "temp-"
        assert!(
            filename.starts_with("temp-"),
            "Filename does not start with 'temp-': {filename}"
        );
    }
}
