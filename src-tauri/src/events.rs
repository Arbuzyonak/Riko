use riko_core::progress::{ProgressEvent, ProgressSink};
use tauri::{AppHandle, Emitter};

pub struct TauriSink {
    app: AppHandle,
    event: &'static str,
}

impl TauriSink {
    pub fn new(app: AppHandle, event: &'static str) -> Self {
        Self { app, event }
    }
}

impl ProgressSink for TauriSink {
    fn emit(&self, event: ProgressEvent) {
        self.app.emit(self.event, event).ok();
    }
}
