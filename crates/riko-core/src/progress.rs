#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    StageStarted {
        stage: String,
        label: String,
    },
    StageProgress {
        stage: String,
        done: u64,
        total: Option<u64>,
    },
    StageLog {
        stage: String,
        level: LogLevel,
        line: String,
    },
    StageFinished {
        stage: String,
        ok: bool,
        detail: Option<String>,
    },
}

pub trait ProgressSink: Send + Sync {
    fn emit(&self, event: ProgressEvent);

    fn started(&self, stage: &str, label: &str) {
        self.emit(ProgressEvent::StageStarted {
            stage: stage.to_string(),
            label: label.to_string(),
        });
    }

    fn progress(&self, stage: &str, done: u64, total: Option<u64>) {
        self.emit(ProgressEvent::StageProgress {
            stage: stage.to_string(),
            done,
            total,
        });
    }

    fn info(&self, stage: &str, line: &str) {
        self.emit(ProgressEvent::StageLog {
            stage: stage.to_string(),
            level: LogLevel::Info,
            line: line.to_string(),
        });
    }

    fn warn(&self, stage: &str, line: &str) {
        self.emit(ProgressEvent::StageLog {
            stage: stage.to_string(),
            level: LogLevel::Warn,
            line: line.to_string(),
        });
    }

    fn error(&self, stage: &str, line: &str) {
        self.emit(ProgressEvent::StageLog {
            stage: stage.to_string(),
            level: LogLevel::Error,
            line: line.to_string(),
        });
    }

    fn finished(&self, stage: &str, ok: bool, detail: Option<String>) {
        self.emit(ProgressEvent::StageFinished {
            stage: stage.to_string(),
            ok,
            detail,
        });
    }
}

pub struct NullSink;

impl ProgressSink for NullSink {
    fn emit(&self, _event: ProgressEvent) {}
}
