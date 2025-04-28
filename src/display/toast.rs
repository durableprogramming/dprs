use std::time::{Duration, Instant};

pub struct Toast {
    pub message: String,
    pub duration: Duration,
    pub created_at: Instant,
}

impl Toast {
    pub fn new(message: &str, duration_ms: u64) -> Self {
        Self {
            message: message.to_string(),
            duration: Duration::from_millis(duration_ms),
            created_at: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }
}

pub struct ToastManager {
    toast: Option<Toast>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self { toast: None }
    }

    pub fn show(&mut self, message: &str, duration_ms: u64) {
        self.toast = Some(Toast::new(message, duration_ms));
    }

    pub fn clear(&mut self) {
        self.toast = None;
    }

    pub fn check_expired(&mut self) {
        if let Some(toast) = &self.toast {
            if toast.is_expired() {
                self.toast = None;
            }
        }
    }

    pub fn get_toast(&self) -> Option<&Toast> {
        self.toast.as_ref()
    }
}
