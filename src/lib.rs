#![allow(dead_code)]
#![allow(unused_macros)]

use lazy_static::lazy_static;
use std::{
    any::Any,
    collections::HashMap,
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

// macro that creates the equivelant of a dictionary in python
/*
Usage:
let mut map = dict!{
    "key1": "value1",
    "key2": "value2",
};
allow any types for the key and value
enforce the trailing comma
*/
macro_rules! dict {
    ($($key:expr => $value:expr),* $(,)+) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert($key, $value);)*
        map
    }};
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

/// Generate temp file name
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
    let hash = format!("{:0>len$}", hash, len = len);

    format!("temp-{}.log", hash)
}

fn get_file_and_filename() -> (Arc<Mutex<File>>, String) {
    let filename: String;
    let file: Arc<Mutex<File>>;
    if !cfg!(test) {
        filename = FILENAME.lock().unwrap().clone();
        file = Arc::new(Mutex::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(filename.to_string())
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
            return ();
        }

        let mut file = self.file.lock().unwrap();
        file.write_all(output.as_bytes()).unwrap();
    }

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

#[derive(Clone)]
pub struct LogInfo {
    pub level: LogLevel,
    pub message: String,
    pub filepath: &'static str,
    pub line_number: u32,
    pub thread: Option<String>,
}

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
            "Contents of log does not contain 'Hello, world!'\nContents: {}",
            contents
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
            "Contents of log does not contain '{}'\nContents: {}\nLogger: {:?}",
            s,
            contents,
            logger
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
            let id = match id {
                Some(id) => id,
                None => 0,
            };
            let message = format!("Hello, world! {}", id);
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
            let message = format!("Hello, world! {}", i);
            check_log_file_contains(message);
        }
    }

    #[test]
    fn test_log_info() {
        let f = function!();
        let s = format!("Hello, {}!", f);
        info!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_debug() {
        let f = function!();
        let s = format!("Hello, {}!", f);
        debug!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_warning() {
        let f = function!();
        let s = format!("Hello, {}!", f);
        warning!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_error() {
        let f = function!();
        let s = format!("Hello, {}!", f);
        error!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_trace() {
        let f = function!();
        let s = format!("Hello, {}!", f);
        trace!(s);
        check_log_file_contains(s);
    }

    #[test]
    fn test_log_text() {
        let f = function!();
        let s = format!("Hello, {}!", f);
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
            "Filename does not start with 'temp-': {}",
            filename
        );
    }

    #[test]
    fn test_dict_macro() {
        let dict1 = dict! {
            "key1" => "value1",
            "key2" => "value2",
        };

        let dict2 = dict! {
            "key1" => "value1",
            "key2" => "value3",
        };

        let dict = dict! {
            "key1" => dict1.clone(),
            "key2" => dict2.clone(),
            "key3" => dict2.clone(),
        };

        assert_eq!(dict1.get("key1"), Some(&"value1"));
        assert_eq!(dict.get("key1"), Some(&dict2));
    }
}

/// Struct similar to a HashMap, but can hold mixed types for values.
#[derive(Debug, Clone, PartialEq)]
pub struct Dict {
    map: HashMap<String, Value>,
}

impl Dict {
    /// Create a new Dict.
    pub fn new() -> Self {
        Dict {
            map: HashMap::new(),
        }
    }

    /// Insert a new key-value pair into the Dict.
    pub fn insert(&mut self, key: &str, value: Value) {
        self.map.insert(key.to_string(), value);
    }

    /// Get the value associated with the given key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key)
    }

    /// Get the value associated with the given key, and convert it to the given type.
    pub fn get_as<T: FromValue>(&self, key: &str) -> Option<T> {
        match self.get(key) {
            Some(value) => Some(T::from_value(value)),
            None => None,
        }
    }

    /// Get the value associated with the given key, and convert it to the given type.
    /// If the value is not found, return the default value.
    pub fn get_as_or<T: FromValue>(&self, key: &str, default: T) -> T {
        match self.get(key) {
            Some(value) => T::from_value(value),
            None => default,
        }
    }

    /// Get the value associated with the given key, and convert it to the given type.
    /// If the value is not found, return the default value.
    pub fn get_as_or_else<T: FromValue, F: FnOnce() -> T>(&self, key: &str, default: F) -> T {
        match self.get(key) {
            Some(value) => T::from_value(value),
            None => default(),
        }
    }

    /// Get the value associated with the given key, and convert it to the given type.
    /// If the value is not found, return the default value.
    pub fn get_as_or_default<T: FromValue + Default>(&self, key: &str) -> T {
        match self.get(key) {
            Some(value) => T::from_value(value),
            None => T::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    value: ValueEnum,
}

impl Value {
    fn new(value: ValueEnum) -> Self {
        Value { value }
    }

    fn as_str(&self) -> Option<&str> {
        match &self.value {
            ValueEnum::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<String> {
        match &self.value {
            ValueEnum::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match &self.value {
            ValueEnum::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match &self.value {
            ValueEnum::I64(i) => Some(*i),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match &self.value {
            ValueEnum::U64(u) => Some(*u),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match &self.value {
            ValueEnum::F64(f) => Some(*f),
            _ => None,
        }
    }

    fn as_dict(&self) -> Option<&Dict> {
        match &self.value {
            ValueEnum::Dict(d) => Some(d),
            _ => None,
        }
    }

    fn as_dict_mut(&mut self) -> Option<&mut Dict> {
        match &mut self.value {
            ValueEnum::Dict(d) => Some(d),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&Vec<Value>> {
        match &self.value {
            ValueEnum::Array(a) => Some(a),
            _ => None,
        }
    }

    fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match &mut self.value {
            ValueEnum::Array(a) => Some(a),
            _ => None,
        }
    }
}

pub trait FromValue {
    fn from_value(value: &Value) -> Self;
}

trait IntoValue {
    fn into_value(self) -> Value;
}

impl FromValue for String {
    fn from_value(value: &Value) -> Self {
        match &value.value {
            ValueEnum::String(s) => s.clone(),
            _ => panic!("Cannot convert {:?} to String", value),
        }
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::new(ValueEnum::String(self))
    }
}

impl IntoValue for Vec<String> {
    fn into_value(self) -> Value {
        Value::new(ValueEnum::Array(
            self.into_iter().map(|s| s.into_value()).collect(),
        ))
    }
}

impl FromValue for Vec<String> {
    fn from_value(value: &Value) -> Self {
        match &value.value {
            ValueEnum::Array(a) => a
                .iter()
                .map(|v| match &v.value {
                    ValueEnum::String(s) => s.clone(),
                    _ => panic!("Cannot convert {:?} to String", v),
                })
                .collect(),
            _ => panic!("Cannot convert {:?} to Vec<String>", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ValueEnum {
    String(String),
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Dict(Dict),
    Array(Vec<Value>),
}

fn get_dict() -> Dict {
    let key1 = String::from("key1");
    let val1 = String::from("value1");
    let key2 = String::from("key2");
    let val2 = vec![String::from("value2")];

    let mut dict1 = Dict::new();
    dict1.insert(&key1, val1.into_value());
    dict1.insert(&key2, val2.into_value());

    dict1
}

#[test]
fn test_get_as() {
    let dict1 = get_dict();
    let val1 = dict1.get_as::<String>("key1").unwrap();
    assert_eq!(val1, "value1");

    let val2 = dict1.get_as::<String>("key2").unwrap();
    assert_eq!(val2, "value2");
}
