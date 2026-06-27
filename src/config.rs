use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ClippyConfig {
    pub position: PositionConfig,
    pub update_interval_seconds: u64,
    pub comment_cooldown_seconds: u64,
    pub bubble_show_seconds: u64,
    pub personality_multiplier: f32,
    pub comment_lists: CommentListsConfig,
    pub bash_history: BashHistoryConfig,
    #[serde(rename = "ai-slop")]
    pub llm: LLMConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PositionConfig {
    pub corner: String,
    pub margin_x: i32,
    pub margin_y: i32,
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            corner: "bottom-right".to_string(),
            margin_x: 0,
            margin_y: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CommentListsConfig {
    pub directory: String,
    pub active: Vec<String>,
}

impl Default for CommentListsConfig {
    fn default() -> Self {
        Self {
            directory: "comments".to_string(),
            active: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct BashHistoryConfig {
    pub enabled: bool,
    pub history_file: Option<PathBuf>,
    pub comment_chance: f32,
    pub poll_interval_seconds: u64,
}

impl Default for BashHistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            history_file: None,
            comment_chance: 0.25,
            poll_interval_seconds: 2,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LLMConfig {
    pub enabled: bool,
    pub api_key: String,
    pub endpoint: String,
    pub model: String,
    pub system_prompt: String,
    pub use_for_window_comments: bool,
    pub use_for_bash_comments: bool,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub screen_vision: bool,
    #[serde(default = "default_screen_vision_scope")]
    pub screen_vision_scope: String,
}

fn default_screen_vision_scope() -> String {
    "window".to_string()
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-4o-mini".to_string(),
            system_prompt: String::new(),
            use_for_window_comments: false,
            use_for_bash_comments: false,
            timeout_seconds: 8,
            screen_vision: false,
            screen_vision_scope: "window".to_string(),
        }
    }
}

impl Default for ClippyConfig {
    fn default() -> Self {
        Self {
            position: PositionConfig::default(),
            update_interval_seconds: 5,
            comment_cooldown_seconds: 40,
            bubble_show_seconds: 7,
            personality_multiplier: 1.0,
            comment_lists: CommentListsConfig::default(),
            bash_history: BashHistoryConfig::default(),
            llm: LLMConfig::default(),
        }
    }
}

impl ClippyConfig {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&config_path) {
            match serde_json::from_str(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!(
                        "clippy-linux: couldn't parse {} ({e}); using defaults instead",
                        config_path.display()
                    );
                    Self::default()
                }
            }
        } else {
            let default_config = Self::default();
            let _ = default_config.save();
            default_config
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(config_path, content)
    }

    pub fn config_dir() -> PathBuf {
        ProjectDirs::from("com", "clippy", "clippy-linux")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".config").join("clippy-linux")
            })
    }

    pub fn comments_dir(&self) -> PathBuf {
        let dir = PathBuf::from(&self.comment_lists.directory);
        if dir.is_absolute() {
            dir
        } else {
            Self::config_dir().join(dir)
        }
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }
}
