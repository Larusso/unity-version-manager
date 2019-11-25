use console::Style;
use flexi_logger::Logger as FLogger;
pub use flexi_logger::{
    Cleanup, Criterion, DeferredNow, Duplicate, FlexiLoggerError, Level, LevelFilter,
    LogSpecification, LogTarget, Naming, ReconfigurationHandle, Record,
};

use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

pub struct Logger {
    inner: FLogger,
    log_dir: Option<PathBuf>,
}

impl Deref for Logger {
    type Target = FLogger;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Logger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Logger {
    pub fn new() -> Self {
        Self::with(uvm_default_log_spec())
    }

    pub fn with(spec: LogSpecification) -> Self {
        let inner = FLogger::with(spec)
            .format_for_files(flexi_logger::detailed_format)
            .format_for_stderr(format_logs)
            .duplicate_to_stderr(Duplicate::None)
            .rotate(
                Criterion::Size(10_000_000),
                Naming::Numbers,
                Cleanup::KeepLogFiles(10),
            );
        Logger {
            inner,
            log_dir: None,
        }
    }

    pub fn log_dir<P: AsRef<Path>>(mut self, directory: P) -> Logger {
        self.log_dir = Some(directory.as_ref().to_path_buf());
        self
    }

    pub fn duplicate_to_stderr(self, duplicate: Duplicate) -> Logger {
        Logger {
            inner: self.inner.duplicate_to_stderr(duplicate),
            log_dir: self.log_dir,
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn start(self) -> Result<ReconfigurationHandle, FlexiLoggerError> {
        if let Some(log_dir) = self.log_dir {
            self.inner.log_target(LogTarget::File).directory(log_dir)
        } else {
            self.inner.log_target(LogTarget::DevNull)
        }
        .start()
    }

    #[cfg(target_os = "linux")]
    pub fn start(self) -> Result<ReconfigurationHandle, FlexiLoggerError> {
        let target = match SyslogConnector::try_datagram("/dev/log")
            .or_else(|_| SyslogConnector::try_datagram("/var/run/syslog"))
            .and_then(|connection| {
                SyslogWriter::try_new(
                    SyslogFacility::LocalUse1,
                    None,
                    LevelFilter::Debug,
                    "uvm-install2".to_string(),
                    syslog_connector,
                )
            }) {
            Err(_) => {
                eprintln("unable to connect to syslog");
                LogTarget::DevNull
            }
            Ok(sys_log_writer) => LogTarget::Writer(sys_log_writer),
        };

        if let Some(log_dir) = log_dir {
            let target = match target {
                LogTarget::Writer(w) => LogTarget::FileAndWriter(w),
                _ => LogTarget::File,
            };
            self.inner.log_target(target).directory(log_dir)
        } else {
            self.inner.log_target(target)
        }
        .start()
    }
}

fn uvm_default_log_spec() -> LogSpecification {
    LogSpecification::default(LevelFilter::Warn)
        .module("uvm_", LevelFilter::Trace)
        .build()
}

fn format_logs(
    writer: &mut dyn io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), io::Error> {
    let style = match record.level() {
        Level::Trace => Style::new().white().dim().italic(),
        Level::Debug => Style::new().white().dim(),
        Level::Info => Style::new().white(),
        Level::Warn => Style::new().yellow(),
        Level::Error => Style::new().red(),
    };

    writer
        .write(&format!("{}", style.apply_to(record.args())).into_bytes())
        .map(|_| ())
}

#[cfg(target_os = "linux")]
pub fn default_log_dir() -> Option<PathBuf> {
    None
}

#[cfg(windows)]
pub fn default_log_dir() -> Option<PathBuf> {
    dirs_2::home_dir().map(|p| p.join(".uvm/logs").to_path_buf())
}

#[cfg(target_os = "macos")]
pub fn default_log_dir() -> Option<PathBuf> {
    dirs_2::home_dir().map(|p| p.join("Library/Logs/UnityVersionManager").to_path_buf())
}
