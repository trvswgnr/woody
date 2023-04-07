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
woody = { git = "https://github.com/trvswgnr/woody.git" }
```

Then, add this to your crate root:

```rust
use woody::*;
```

## Examples

### Basic

```rust
fn main() {
    log!(Info, "An info message.");
    debug!("A debug message.");
    info!("An info message.");
    warn!("A warning message.");
    error!("An error message.");
    trace!("A trace message.");
}
```

Logs are output to the `debug.log` file in the current directory.
