# flexi_logger_rotate_writer

A custom log writer for emabee's [flexi_logger](https://github.com/emabee/flexi_logger). It is just a simplified version of flexi_logger's `FileLogWriter`. Simply rotates every day. No cleanup. No other configs.

## Example usage

```rust
use flexi_logger_rotate_writer::RotateLogWriter;
use flexi_logger::{Logger, LogTarget};

fn main() {
    let log_writer = RotateLogWriter::builder()
        .directory("path/to/where/you/want/to/store/the/log/files")
        // Some other configs...
        .try_build()?;

    Logger::with_env()
        .log_target(LogTarget::Writer(Box::new(log_writer)))
        // Some other configs...
        .start()?;

    // ...
}
```
