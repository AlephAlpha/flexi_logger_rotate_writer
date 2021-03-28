//! A custom log writer for emabee's [flexi_logger](https://github.com/emabee/flexi_logger).
//!
//! It is just a simplified version of flexi_logger's
//! [`FileLogWriter`](https://docs.rs/flexi_logger/0.17.1/flexi_logger/writers/struct.FileLogWriter.html).
//! Simply rotates every day, and stores the logs in files like `foo_r2021-03-28.log`.
//! No cleanup. No other configs.
//!
//! ## Example usage
//! ```rust
//! use flexi_logger_rotate_writer::RotateLogWriter;
//! use flexi_logger::{Logger, LogTarget};
//!
//! let log_writer = RotateLogWriter::builder()
//!     .directory("path/to/where/you/want/to/store/the/log/files")
//!     // Some other configs...
//!     .try_build()
//!     .unwrap();
//!
//! Logger::with_env()
//!     .log_target(LogTarget::Writer(Box::new(log_writer)))
//!     // Some other configs...
//!     .start()
//!     .unwrap();
//!
//! // ...
//! ```

use flexi_logger::{writers::LogWriter, DeferredNow, FormatFunction, LevelFilter, Record};
use state::State;
use std::{
    cell::RefCell,
    io::{Result as IoResult, Write},
    sync::Mutex,
};

mod builder;
mod config;
mod state;

pub use builder::RotateLogWriterBuilder;

const WINDOWS_LINE_ENDING: &[u8] = b"\r\n";
const UNIX_LINE_ENDING: &[u8] = b"\n";

/// A simplified version of `flexi_logger`'s
/// [`FileLogWriter`](https://docs.rs/flexi_logger/0.17.1/flexi_logger/writers/struct.FileLogWriter.html).
///
/// It simply rotates every day, and stores the logs in files like `foo_r2021-03-28.log`.
/// No cleanup. No other configs.
pub struct RotateLogWriter {
    format: FormatFunction,
    line_ending: &'static [u8],
    state: Mutex<State>,
    max_log_level: LevelFilter,
}

impl RotateLogWriter {
    pub(crate) fn new(
        format: FormatFunction,
        line_ending: &'static [u8],
        state: Mutex<State>,
        max_log_level: LevelFilter,
    ) -> Self {
        Self {
            format,
            line_ending,
            state,
            max_log_level,
        }
    }

    /// Instantiates a builder for [`RotateLogWriter`].
    #[must_use]
    pub fn builder() -> RotateLogWriterBuilder {
        RotateLogWriterBuilder::default()
    }
}

impl LogWriter for RotateLogWriter {
    #[inline]
    fn write(&self, now: &mut DeferredNow, record: &Record) -> IoResult<()> {
        thread_local! {
            static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(200));
        }
        BUFFER.with(|tl_buf| match tl_buf.try_borrow_mut() {
            Ok(mut buffer) => {
                (self.format)(&mut *buffer, now, record).unwrap_or_else(|e| write_err(ERR_1, &e));

                let mut state = self.state.lock().unwrap();

                buffer
                    .write_all(self.line_ending)
                    .unwrap_or_else(|e| write_err(ERR_2, &e));

                state
                    .write_buffer(&*buffer)
                    .unwrap_or_else(|e| write_err(ERR_2, &e));
                buffer.clear();
            }
            Err(_e) => {
                // We arrive here in the rare cases of recursive logging
                // (e.g. log calls in Debug or Display implementations)
                // we print the inner calls, in chronological order, before finally the
                // outer most message is printed
                let mut tmp_buf = Vec::<u8>::with_capacity(200);
                (self.format)(&mut tmp_buf, now, record).unwrap_or_else(|e| write_err(ERR_1, &e));

                let mut state = self.state.lock().unwrap();

                tmp_buf
                    .write_all(self.line_ending)
                    .unwrap_or_else(|e| write_err(ERR_2, &e));

                state
                    .write_buffer(&tmp_buf)
                    .unwrap_or_else(|e| write_err(ERR_2, &e));
            }
        });
        Ok(())
    }

    #[inline]
    fn flush(&self) -> IoResult<()> {
        if let Ok(mut state) = self.state.lock() {
            state.flush()
        } else {
            Ok(())
        }
    }

    #[inline]
    fn max_log_level(&self) -> LevelFilter {
        self.max_log_level
    }

    #[inline]
    fn format(&mut self, format: FormatFunction) {
        self.format = format;
    }
}

const ERR_1: &str = "FileLogWriter: formatting failed with ";
const ERR_2: &str = "FileLogWriter: writing failed with ";

fn write_err(msg: &str, err: &std::io::Error) {
    eprintln!("[flexi_logger] {} with {}", msg, err);
}
