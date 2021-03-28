use crate::{config::Config, state::State, RotateLogWriter};
use flexi_logger::{default_format, FlexiLoggerError, FormatFunction, LevelFilter};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

/// Builder for [`RotateLogWriter`].
pub struct RotateLogWriterBuilder {
    basename: Option<String>,
    discriminant: Option<String>,
    config: Config,
    format: FormatFunction,
    max_log_level: LevelFilter,
}

impl Default for RotateLogWriterBuilder {
    fn default() -> Self {
        Self {
            basename: None,
            discriminant: None,
            config: Config::default(),
            format: default_format,
            max_log_level: LevelFilter::Trace,
        }
    }
}

impl RotateLogWriterBuilder {
    /// Makes the [`RotateLogWriter`] print an info message to stdout
    /// when a new file is used for log-output.
    #[inline]
    #[must_use]
    pub const fn print_message(mut self) -> Self {
        self.config.print_message = true;
        self
    }

    /// Makes the [`RotateLogWriter`] use the provided format function for the log entries,
    /// rather than the [`default_format`](https://docs.rs/flexi_logger/0.17.1/flexi_logger/fn.default_format.html).
    #[inline]
    #[must_use]
    pub fn format(mut self, format: FormatFunction) -> Self {
        self.format = format;
        self
    }

    /// Specifies a folder for the log files.
    ///
    /// If the specified folder does not exist, the initialization will fail.
    /// By default, the log files are created in the folder where the program was started.
    #[inline]
    #[must_use]
    pub fn directory<P: Into<PathBuf>>(mut self, directory: P) -> Self {
        self.config.filename_config.directory = directory.into();
        self
    }

    /// Specifies a suffix for the log files. The default is "log".
    #[inline]
    #[must_use]
    pub fn suffix<S: Into<String>>(mut self, suffix: S) -> Self {
        self.config.filename_config.suffix = suffix.into();
        self
    }

    /// The specified String is added to the log file name.
    #[inline]
    #[must_use]
    pub fn discriminant<S: Into<String>>(mut self, discriminant: S) -> Self {
        self.discriminant = Some(discriminant.into());
        self
    }

    /// The specified String is used as the basename of the log file name,
    /// instead of the program name.
    #[inline]
    #[must_use]
    pub fn basename<S: Into<String>>(mut self, basename: S) -> Self {
        self.basename = Some(basename.into());
        self
    }

    /// The specified String will be used on linux systems to create in the current folder
    /// a symbolic link to the current log file.
    #[inline]
    #[must_use]
    pub fn create_symlink<P: Into<PathBuf>>(mut self, symlink: P) -> Self {
        self.config.o_create_symlink = Some(symlink.into());
        self
    }

    /// Use Windows line endings, rather than just `\n`.
    #[inline]
    #[must_use]
    pub const fn use_windows_line_ending(mut self) -> Self {
        self.config.line_ending = super::WINDOWS_LINE_ENDING;
        self
    }

    /// Define if buffering should be used.
    ///
    /// By default, every log line is directly written to the output file, without buffering.
    /// This allows seeing new log lines in real time.
    #[inline]
    #[must_use]
    pub const fn use_buffering(mut self, buffer: bool) -> Self {
        if buffer {
            self.config.o_buffersize = Some(8 * 1024);
        } else {
            self.config.o_buffersize = None;
        }
        self
    }

    /// Activates buffering, and uses a buffer with the specified capacity.
    #[inline]
    #[must_use]
    pub const fn buffer_with_capacity(mut self, capacity: usize) -> Self {
        self.config.o_buffersize = Some(capacity);
        self
    }

    /// Produces the [`RotateLogWriter`].
    pub fn try_build(mut self) -> Result<RotateLogWriter, FlexiLoggerError> {
        // make sure the folder exists or create it
        let p_directory = Path::new(&self.config.filename_config.directory);
        std::fs::create_dir_all(&p_directory)?;
        if !std::fs::metadata(&p_directory)?.is_dir() {
            return Err(FlexiLoggerError::OutputBadDirectory);
        };

        if let Some(basename) = self.basename {
            self.config.filename_config.file_basename = basename;
        } else {
            let arg0 = std::env::args().next().unwrap_or_else(|| "rs".to_owned());
            self.config.filename_config.file_basename =
                Path::new(&arg0).file_stem().unwrap(/*cannot fail*/).to_string_lossy().to_string();
        }

        if let Some(discriminant) = self.discriminant {
            self.config.filename_config.file_basename += &format!("_{}", discriminant);
        }

        Ok(RotateLogWriter::new(
            self.format,
            self.config.line_ending,
            Mutex::new(State::new(self.config)),
            self.max_log_level,
        ))
    }
}
