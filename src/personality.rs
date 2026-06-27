use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::config::ClippyConfig;
use crate::llm::LLMClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistroClass {
    Elite,
    Middle,
    Worker,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Window,
    BashCommand,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommentList {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub animation: Option<String>,
    #[serde(default)]
    pub comments: Vec<String>,
}

pub struct Personality {
    class: DistroClass,
    lists: Vec<CommentList>,
    llm_client: LLMClient,
    llm_enabled: bool,
    llm_use_for_window: bool,
    llm_use_for_bash: bool,
    llm_timeout_secs: u64,
    llm_screen_vision: bool,
    llm_screen_vision_scope: String,
}

impl Personality {
    pub fn new(config: &ClippyConfig) -> Self {
        let class = Self::detect_distro();
        let comments_dir = config.comments_dir();

        Self::ensure_default_lists(&comments_dir, class);
        let mut lists = Self::load_lists(&comments_dir);

        if !config.comment_lists.active.is_empty() {
            let active: std::collections::HashSet<&str> = config
                .comment_lists
                .active
                .iter()
                .map(|s| s.as_str())
                .collect();
            lists.retain(|l| active.contains(l.name.as_str()));
        }

        Self {
            class,
            lists,
            llm_client: LLMClient::new(config.llm.clone()),
            llm_enabled: config.llm.enabled,
            llm_use_for_window: config.llm.use_for_window_comments,
            llm_use_for_bash: config.llm.use_for_bash_comments,
            llm_timeout_secs: config.llm.timeout_seconds.max(1),
            llm_screen_vision: config.llm.screen_vision,
            llm_screen_vision_scope: config.llm.screen_vision_scope.clone(),
        }
    }

    fn detect_distro() -> DistroClass {
        let os_release = fs::read_to_string("/etc/os-release").unwrap_or_default();
        if os_release.contains("Gentoo") || os_release.contains("Linux From Scratch") {
            DistroClass::Elite
        } else if os_release.contains("Arch") {
            DistroClass::Middle
        } else if os_release.contains("Debian")
            || os_release.contains("Fedora")
            || os_release.contains("Red Hat")
        {
            DistroClass::Worker
        } else {
            DistroClass::Bottom
        }
    }

    pub async fn comment_for(&self, context: &str, kind: CommentKind) -> (String, String) {
        let use_llm = self.llm_enabled
            && match kind {
                CommentKind::Window => self.llm_use_for_window,
                CommentKind::BashCommand => self.llm_use_for_bash,
            };

        if use_llm {
            let screenshot = if self.llm_screen_vision {
                match self.capture_screen_image(&self.llm_screen_vision_scope).await {
                    Ok(bytes) => Some(bytes),
                    Err(e) => {
                        eprintln!("clippy-linux: screenshot failed: {e}");
                        None
                    }
                }
            } else {
                None
            };

            let flavor = self.flavor_description();
            let outcome = tokio::time::timeout(
                Duration::from_secs(self.llm_timeout_secs),
                self.llm_client.get_dynamic_comment(context, &flavor, screenshot),
            )
            .await;

            match outcome {
                Ok(Ok(text)) if !text.trim().is_empty() => {
                    return (text.trim().to_string(), self.suggest_animation(context));
                }
                Ok(Err(e)) => eprintln!("clippy-linux: LLM request failed: {e}"),
                Err(_) => eprintln!(
                    "clippy-linux: LLM request timed out after {}s, falling back to comment lists",
                    self.llm_timeout_secs
                ),
                _ => {}
            }
        }

        self.static_comment_for(context)
    }

    pub fn static_comment_for(&self, context: &str) -> (String, String) {
        let mut rng = rand::thread_rng();

        let matched = self.matching_lists(context);
        if !matched.is_empty() {
            if let Some(list) = matched.choose(&mut rng) {
                if let Some(comment) = list.comments.choose(&mut rng) {
                    let animation = list
                        .animation
                        .clone()
                        .unwrap_or_else(|| self.suggest_animation(context));
                    return (comment.replace("%s", context), animation);
                }
            }
        }

        let general: Vec<&String> = self
            .lists
            .iter()
            .filter(|l| l.triggers.is_empty())
            .flat_map(|l| l.comments.iter())
            .collect();
        if let Some(c) = general.choose(&mut rng) {
            return ((*c).replace("%s", context), self.suggest_animation(context));
        }

        ("RTFM.".to_string(), "Idle1_1".to_string())
    }

    fn matching_lists(&self, context: &str) -> Vec<&CommentList> {
        let ctx = context.to_lowercase();
        self.lists
            .iter()
            .filter(|l| {
                !l.triggers.is_empty()
                    && l.triggers
                        .iter()
                        .any(|t| !t.trim().is_empty() && ctx.contains(&t.to_lowercase()))
            })
            .collect()
    }

    fn suggest_animation(&self, context: &str) -> String {
        let ctx = context.to_lowercase();
        if ctx.contains("sudo") {
            "Alert".to_string()
        } else if ctx.contains("vim") || ctx.contains("nvim") || ctx.contains("emacs") {
            "Thinking".to_string()
        } else if ctx.contains("doas") {
            "Congratulate".to_string()
        } else {
            "Idle1_1".to_string()
        }
    }

    fn flavor_description(&self) -> String {
        match self.class {
            DistroClass::Elite => "You're talking to a Gentoo/LFS user who compiles everything from source. Grudging, cold respect.".to_string(),
            DistroClass::Middle => "You're talking to an Arch Linux user. Be condescending about their superiority complex.".to_string(),
            DistroClass::Worker => "You're talking to a Debian/Fedora/RHEL user. Be bored, dismissive, treat them like corporate IT furniture.".to_string(),
            DistroClass::Bottom => "You're talking to an Ubuntu/Mint/Manjaro user. Be furious and patronizing, like they're a child with a toy computer.".to_string(),
        }
    }

    fn load_lists(dir: &Path) -> Vec<CommentList> {
        let mut lists = Vec::new();
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return lists,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("clippy-linux: couldn't read {} ({e})", path.display());
                    continue;
                }
            };
            match serde_json::from_str::<CommentList>(&content) {
                Ok(mut list) => {
                    if list.name.is_empty() {
                        list.name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unnamed")
                            .to_string();
                    }
                    if !list.comments.is_empty() {
                        lists.push(list);
                    }
                }
                Err(e) => eprintln!(
                    "clippy-linux: skipping {} (invalid format: {e})",
                    path.display()
                ),
            }
        }
        lists
    }

    fn ensure_default_lists(dir: &Path, class: DistroClass) {
        let is_empty = match fs::read_dir(dir) {
            Ok(mut entries) => entries.next().is_none(),
            Err(_) => true,
        };
        if !is_empty {
            return;
        }
        let _ = fs::create_dir_all(dir);

        for (filename, list) in Self::default_lists(class) {
            let path = dir.join(filename);
            if path.exists() {
                continue;
            }
            match serde_json::to_string_pretty(&list) {
                Ok(json) => {
                    if let Err(e) = fs::write(&path, json) {
                        eprintln!(
                            "clippy-linux: couldn't write default list {} ({e})",
                            path.display()
                        );
                    }
                }
                Err(e) => {
                    eprintln!("clippy-linux: couldn't serialize default list {filename} ({e})")
                }
            }
        }
    }

    fn default_lists(class: DistroClass) -> Vec<(&'static str, CommentList)> {
        let mk = |name: &str, triggers: &[&str], animation: Option<&str>, comments: &[&str]| {
            CommentList {
                name: name.to_string(),
                triggers: triggers.iter().map(|s| s.to_string()).collect(),
                animation: animation.map(|s| s.to_string()),
                comments: comments.iter().map(|s| s.to_string()).collect(),
            }
        };

        let distro_comments: &[&str] = match class {
            DistroClass::Elite => &[
                "Efficient. I respect that.",
                "Finally someone who compiles their own kernel.",
                "Acceptable. Keep it up.",
                "I see you still have all your hair. Impressive.",
            ],
            DistroClass::Middle => &[
                "Oh look, another Arch user. How original.",
                "I bet you've told 5 people you use Arch today.",
                "AUR is just a crutch for the lazy.",
                "By the way, I use Arch.",
            ],
            DistroClass::Worker => &[
                "Predictable. Boring. Stable. Yawn.",
                "You're just a cog in the corporate IT machine.",
                "Is there any excitement in your life, or just .deb packages?",
                "systemd: because who needs simplicity.",
            ],
            DistroClass::Bottom => &[
                "Is this a toy computer?",
                "I can smell the bloated GUI from here.",
                "Ubuntu? Just uninstall everything and start over.",
                "Using a GUI? How adorable.",
            ],
        };

        vec![
            ("general.json", mk("general", &[], None, &[
                "RTFM.",
                "I've seen better configs written by a cat walking on a keyboard.",
                "Did you mean to do that, or is this just how you live?",
                "Fascinating. Anyway.",
            ])),
            ("distro.json", mk("distro", &[], None, distro_comments)),
            ("vim.json", mk("vim", &["vim", "nvim"], Some("Thinking"), &[
                "Vim. The only way to live. Try exiting without a tutorial.",
                "Modal editing. Bold choice for someone who still hits Ctrl+S out of habit.",
                ":wq and you still got it wrong, didn't you.",
            ])),
            ("emacs.json", mk("emacs", &["emacs"], Some("Thinking"), &[
                "A great OS. Shame it pretends to be a text editor.",
                "Emacs. At this point just admit you're running an operating system.",
            ])),
            ("nano.json", mk("nano", &["nano", "micro"], None, &[
                "Training wheels. Pathetic.",
                "Nano. The text editor equivalent of a participation trophy.",
            ])),
            ("vscode.json", mk("vscode", &["code", "vscode"], None, &[
                "Electron? Burning RAM for a fancy notepad. Disgusting.",
                "A whole web browser just to edit text files. Efficient, truly.",
            ])),
            ("git.json", mk("git", &["git"], None, &[
                "git add .; git commit -m 'fix'; git push. Real programmers squash.",
                "Force-pushing to main again, I see.",
                "Another commit message that explains nothing. Beautiful.",
            ])),
            ("htop.json", mk("htop", &["htop", "top"], None, &[
                "Watching processes instead of fixing them. Classic.",
                "Staring at htop won't lower your CPU usage. Have you tried closing Chrome?",
            ])),
            ("docker.json", mk("docker", &["docker"], None, &[
                "Congrats on reinventing chroot with more bloat.",
                "Just ship the whole OS, why not.",
            ])),
            ("python.json", mk("python", &["python", "python3"], None, &[
                "Python. The BASIC of our generation.",
                "Indentation as syntax. What could possibly go wrong.",
            ])),
            ("sudo.json", mk("sudo", &["sudo"], Some("Alert"), &[
                "Sudo? Why do you still use this bloated relic?",
                "Another one reaching for sudo like it's candy.",
                "Root privileges for THAT command? Bold.",
            ])),
            ("doas.json", mk("doas", &["doas"], Some("Congratulate"), &[
                "doas. Elegant. Pure. You might actually know what you're doing.",
                "Finally, someone who read past page one of the manual.",
            ])),
        ]
    }

    async fn capture_screen_image(&self, _scope: &str) -> Result<Vec<u8>, String> {
        use ashpd::desktop::screenshot::Screenshot;

        let response = Screenshot::request()
            .modal(false)
            .interactive(false)
            .send()
            .await
            .map_err(|e| format!("Failed to send screenshot request: {e}"))?
            .response()
            .map_err(|e| format!("Screenshot request failed: {e}"))?;

        let uri = response.uri().to_string();
        let path = uri
            .strip_prefix("file://")
            .ok_or_else(|| "Unexpected URI scheme".to_string())?;
        
        let bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read screenshot: {e}"))?;
            
        let _ = std::fs::remove_file(path);
        
        Ok(bytes)
    }
}
