use crate::config::{Config, FilenameConfig};
use chrono::{Date, Datelike, Local};
use std::{
    fs::OpenOptions,
    io::{BufWriter, Result as IoResult, Write},
    path::PathBuf,
};

struct RotationState {
    created_at: Date<Local>,
}

impl RotationState {
    fn rotation_necessary(&self) -> bool {
        let today = Local::today();
        self.created_at.num_days_from_ce() != today.num_days_from_ce()
    }
}

enum Inner {
    Initial,
    Active(RotationState, Box<dyn Write + Send>),
}

/// The mutable state of a `RotateLogWriter`.
pub struct State {
    config: Config,
    inner: Inner,
}

impl State {
    pub(crate) const fn new(config: Config) -> Self {
        Self {
            inner: Inner::Initial,
            config,
        }
    }

    fn initialize(&mut self) -> IoResult<()> {
        if let Inner::Initial = &self.inner {
            let (log_file, created_at) = open_log_file(&self.config)?;
            self.inner = Inner::Active(RotationState { created_at }, log_file);
        }
        Ok(())
    }

    pub(crate) fn flush(&mut self) -> IoResult<()> {
        if let Inner::Active(_, file) = &mut self.inner {
            file.flush()
        } else {
            Ok(())
        }
    }

    #[inline]
    fn mount_next_linewriter_if_necessary(&mut self) -> IoResult<()> {
        if let Inner::Active(rotation_state, file) = &mut self.inner {
            if rotation_state.rotation_necessary() {
                let (log_file, created_at) = open_log_file(&self.config)?;
                *file = log_file;
                rotation_state.created_at = created_at;
            }
        }
        Ok(())
    }

    pub(crate) fn write_buffer(&mut self, buf: &[u8]) -> IoResult<()> {
        self.initialize()?;
        // rotate if necessary
        self.mount_next_linewriter_if_necessary()
            .unwrap_or_else(|e| {
                eprintln!("[flexi_logger] opening file failed with {}", e);
            });

        if let Inner::Active(_rotation_state, log_file) = &mut self.inner {
            log_file.write_all(buf)?;
        }
        Ok(())
    }
}

fn get_filepath(date: Date<Local>, config: &FilenameConfig) -> PathBuf {
    let date_infix = date.format("%Y-%m-%d").to_string();
    let s_filename = format!("{}_r{}.{}", config.file_basename, date_infix, config.suffix);
    let mut p_path = config.directory.to_path_buf();
    p_path.push(s_filename);
    p_path
}

fn open_log_file(config: &Config) -> IoResult<(Box<dyn Write + Send>, Date<Local>)> {
    let today = Local::today();
    let p_path = get_filepath(today, &config.filename_config);
    if config.print_message {
        println!("Log is written to {}", &p_path.display());
    }
    #[cfg(target_os = "linux")]
    if let Some(ref link) = config.o_create_symlink {
        self::linux::create_symlink(link, &p_path);
    }
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&p_path)?;
    let w: Box<dyn Write + Send> = if let Some(capacity) = config.o_buffersize {
        Box::new(BufWriter::with_capacity(capacity, log_file))
    } else {
        Box::new(log_file)
    };
    Ok((w, today))
}

#[cfg(target_os = "linux")]
mod linux {
    use std::path::Path;

    pub fn create_symlink(link: &Path, logfile: &Path) {
        if std::fs::symlink_metadata(link).is_ok() {
            // remove old symlink before creating a new one
            if let Err(e) = std::fs::remove_file(link) {
                eprintln!(
                    "[flexi_logger] deleting old symlink to log file failed with {:?}",
                    e
                );
            }
        }

        // create new symlink
        if let Err(e) = std::os::unix::fs::symlink(&logfile, link) {
            eprintln!(
                "[flexi_logger] cannot create symlink {:?} for logfile \"{}\" due to {:?}",
                link,
                &logfile.display(),
                e
            );
        }
    }
}
