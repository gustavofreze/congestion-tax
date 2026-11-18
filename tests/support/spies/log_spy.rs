use std::io::{Result, Write};
use std::sync::{Arc, Mutex, PoisonError};

use tracing::Subscriber;
use tracing_subscriber::fmt;

#[derive(Clone)]
pub(crate) struct LogSpy {
    written: Arc<Mutex<Vec<u8>>>,
}

impl LogSpy {
    pub(crate) fn listening() -> Self {
        Self {
            written: Arc::default(),
        }
    }

    pub(crate) fn subscriber(&self) -> impl Subscriber + Send + Sync + 'static {
        let log = self.clone();

        fmt()
            .with_ansi(false)
            .with_writer(move || log.clone())
            .finish()
    }

    pub(crate) fn logged_lines(&self) -> Vec<String> {
        let written = self.written.lock().unwrap_or_else(PoisonError::into_inner);

        String::from_utf8_lossy(&written).lines().map(str::to_owned).collect()
    }
}

impl Write for LogSpy {
    fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        self.written
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(buffer);

        Ok(buffer.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
