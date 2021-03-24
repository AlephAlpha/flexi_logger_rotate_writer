use crate::UNIX_LINE_ENDING;
use std::path::PathBuf;

#[derive(Clone)]
pub struct FilenameConfig {
    pub(crate) directory: PathBuf,
    pub(crate) file_basename: String,
    pub(crate) suffix: String,
}

/// The immutable configuration of a `RotateLogWriter`.
pub struct Config {
    pub(crate) print_message: bool,
    pub(crate) o_buffersize: Option<usize>,
    pub(crate) filename_config: FilenameConfig,
    pub(crate) o_create_symlink: Option<PathBuf>,
    pub(crate) line_ending: &'static [u8],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            print_message: false,
            filename_config: FilenameConfig {
                directory: PathBuf::from("."),
                file_basename: String::new(),
                suffix: "log".to_string(),
            },
            o_buffersize: None,
            o_create_symlink: None,
            line_ending: UNIX_LINE_ENDING,
        }
    }
}
