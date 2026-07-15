use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

static LOGGER: LazyLock<Mutex<Option<LogFile>>> = LazyLock::new(|| Mutex::new(None));

struct LogFile {
    path: PathBuf,
}

impl LogFile {
    fn append(&self, msg: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            writeln!(file, "{msg}").ok();
        }
    }
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn init(path: PathBuf) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let mut guard = LOGGER.lock().expect("logger lock");
    *guard = Some(LogFile { path });
}

fn append(level: &str, msg: &str) {
    let entry = format!("[{}] [{}] {}", timestamp(), level, msg);
    let guard = LOGGER.lock().expect("logger lock");
    if let Some(log) = guard.as_ref() {
        log.append(&entry);
    }
}

pub fn error(msg: &str) {
    append("ERROR", msg);
}

pub fn warn(msg: &str) {
    append("WARN", msg);
}

pub fn info(msg: &str) {
    append("INFO", msg);
}
