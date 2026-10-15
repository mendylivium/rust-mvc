use crate::app::http::error::{AppError, Result};
use axum::http::StatusCode;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Shared by all router clones. No attacker-controlled keys or unbounded queues.
#[derive(Clone)]
pub struct Security {
    logins: Arc<Mutex<Window>>,
    sessions: Arc<Mutex<Window>>,
    passwords: Arc<Semaphore>,
}
struct Window {
    start: Instant,
    count: u32,
}
impl Default for Security {
    fn default() -> Self {
        Self {
            logins: Arc::new(Mutex::new(Window {
                start: Instant::now(),
                count: 0,
            })),
            sessions: Arc::new(Mutex::new(Window {
                start: Instant::now(),
                count: 0,
            })),
            passwords: Arc::new(Semaphore::new(4)),
        }
    }
}
fn limited() -> AppError {
    AppError::new(
        StatusCode::TOO_MANY_REQUESTS,
        "Too many requests. Please try again shortly.",
    )
}
impl Security {
    fn admit(window: &Mutex<Window>, max: u32) -> Result<()> {
        let mut window = window.lock().unwrap_or_else(|e| e.into_inner());
        if window.start.elapsed() >= Duration::from_secs(60) {
            window.start = Instant::now();
            window.count = 0;
        }
        if window.count >= max {
            return Err(limited());
        }
        window.count += 1;
        Ok(())
    }
    pub fn login(&self) -> Result<()> {
        Self::admit(&self.logins, 30)
    }
    pub fn session(&self) -> Result<()> {
        Self::admit(&self.sessions, 60)
    }
    pub fn password(&self) -> Result<OwnedSemaphorePermit> {
        self.passwords
            .clone()
            .try_acquire_owned()
            .map_err(|_| limited())
    }
}
