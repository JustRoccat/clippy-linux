use std::env;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

pub struct SystemMonitor {
    pub last_process: String,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            last_process: String::new(),
        }
    }

    pub async fn monitor_loop<F>(poll_interval: Duration, mut callback: F)
    where
        F: FnMut(String, bool) + Send + 'static,
    {
        let mut monitor = Self::new();
        loop {
            if let Some(current_process) = monitor.get_active_process() {
                if current_process != monitor.last_process {
                    callback(current_process.clone(), true);
                    monitor.last_process = current_process;
                }
            }
            sleep(poll_interval).await;
        }
    }

    fn get_active_process(&self) -> Option<String> {
        if env::var("WAYLAND_DISPLAY").is_ok() {
            return self.get_wayland_process();
        }

        self.get_x11_process()
            .or_else(|| self.get_fallback_process())
    }

    fn get_wayland_process(&self) -> Option<String> {
        if let Ok(out) = Command::new("hyprctl").args(["activewindow"]).output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = stdout.lines().find(|l| l.contains("class:")) {
                return Some(line.split(':').last()?.trim().to_string());
            }
        }

        if let Ok(out) = Command::new("swaymsg").args(["-t", "get_focused"]).output() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            if let Some(start) = stdout.find("\"app_id\":") {
                let rest = stdout[start + 9..].trim_start();
                if rest.starts_with('"') {
                    let inner = &rest[1..];
                    if let Some(end) = inner.find('"') {
                        let app_id = &inner[..end];
                        if !app_id.is_empty() && app_id != "null" {
                            return Some(app_id.to_string());
                        }
                    }
                }
            }

            if let Some(start) = stdout.find("\"name\":") {
                let rest = stdout[start + 7..].trim_start();
                if rest.starts_with('"') {
                    let inner = &rest[1..];
                    if let Some(end) = inner.find('"') {
                        let name = &inner[..end];
                        if !name.is_empty() && name != "null" {
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }

        self.get_fallback_process()
    }

    fn get_x11_process(&self) -> Option<String> {
        let output = Command::new("xprop")
            .args(["-root", "_NET_ACTIVE_WINDOW"])
            .output();

        if let Ok(out) = output {
            let window_id = String::from_utf8_lossy(&out.stdout);
            if window_id.is_empty() {
                return None;
            }

            let window_title = Command::new("xprop")
                .args([
                    "-id",
                    window_id.trim().split_whitespace().last().unwrap_or(""),
                    "_NET_WM_NAME",
                ])
                .output();

            if let Ok(title_out) = window_title {
                let title = String::from_utf8_lossy(&title_out.stdout);
                return Some(title.to_string());
            }
        }
        None
    }

    fn get_fallback_process(&self) -> Option<String> {
        let ps_output = Command::new("ps")
            .args(["-eo", "comm", "--sort=-%cpu"])
            .output();

        if let Ok(out) = ps_output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let first_line = stdout.lines().nth(1);
            if let Some(line) = first_line {
                return Some(line.trim().to_string());
            }
        }
        None
    }
}

pub struct BashHistoryWatcher {
    path: PathBuf,
    last_pos: u64,
}

impl BashHistoryWatcher {
    pub fn new(custom_path: Option<PathBuf>) -> Option<Self> {
        let path = custom_path.or_else(Self::detect_path)?;
        let last_pos = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        Some(Self { path, last_pos })
    }

    fn detect_path() -> Option<PathBuf> {
        if let Ok(p) = env::var("HISTFILE") {
            if !p.trim().is_empty() {
                return Some(PathBuf::from(p));
            }
        }
        let home = env::var("HOME").ok()?;
        let candidate = PathBuf::from(home).join(".bash_history");
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    }

    pub async fn watch_loop<F>(
        mut self,
        poll_interval: Duration,
        comment_chance: f32,
        mut callback: F,
    ) where
        F: FnMut(String) + Send + 'static,
    {
        let chance = comment_chance.clamp(0.0, 1.0);
        loop {
            sleep(poll_interval).await;
            for cmd in self.poll_new_commands() {
                if rand::random::<f32>() < chance {
                    callback(cmd);
                }
            }
        }
    }

    fn poll_new_commands(&mut self) -> Vec<String> {
        let len = match std::fs::metadata(&self.path) {
            Ok(m) => m.len(),
            Err(_) => return Vec::new(),
        };

        if len < self.last_pos {
            self.last_pos = 0;
        }
        if len == self.last_pos {
            return Vec::new();
        }

        let mut file = match std::fs::File::open(&self.path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };
        if file.seek(SeekFrom::Start(self.last_pos)).is_err() {
            return Vec::new();
        }
        let mut buf = String::new();
        if file.read_to_string(&mut buf).is_err() {
            return Vec::new();
        }
        self.last_pos = len;

        buf.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .last()
            .map(|l| vec![l.to_string()])
            .unwrap_or_default()
    }
}
