# Woody

A logger for Rust that's \*actually\* easy to use.

## Features

-   **Easy to use:** Just import the macros and you're good to go. No need to
    configure anything. No need to create a logger. Just log.
-   **Versatile:** Log messages at different levels, works across threads, and
    can be used in libraries.
-   **Lightweight:** Relies only on `lazy_static` for thread safety and
    `chrono` for timestamps (in addition to the standard library).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
woody = "0.1.2"
```

## Examples

```rust
use woody::*;

fn main() {
    log!(LogLevel::Info, "An info message.");
    log_debug!("A debug message.");
    log_info!("An info message.");
    log_warn!("A warning message.");
    log_error!("An error message.");
    log_trace!("A trace message.");
}
```

Logs are output to the `woody.log` file in the current directory.

Environment variables can be set to control the log level and output file:

```bash
$ WOODY_LEVEL=error cargo run # Only error messages will be logged
$ WOODY_FILE=woodyrulez.log cargo run # Logs will be written to woodyrulez.log
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to
discuss what you would like to change.

> [!IMPORTANT]  
> When running tests, make sure to remove the `woody.log` file in the current directory after each test run.
> ```shell
> cargo test && rm ./woody.log
> ```

To publish a new version, update the version number in `Cargo.toml` and in `README.md`, and then run:

```shell
cargo publish
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.