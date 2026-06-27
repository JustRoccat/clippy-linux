mod assets;
mod config;
mod llm;
mod monitor;
mod personality;
mod ui;
mod wayland;

use assets::AssetManager;
use config::ClippyConfig;
use monitor::{BashHistoryWatcher, SystemMonitor};
use personality::{CommentKind, Personality};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use ui::AppState;

fn main() {
    let config = ClippyConfig::load();
    let personality = Arc::new(Personality::new(&config));
    let assets = Arc::new(AssetManager::load());
    let state = Arc::new(Mutex::new(AppState::new(
        Duration::from_secs(config.comment_cooldown_seconds.max(1)),
        Duration::from_secs(config.bubble_show_seconds.max(1)),
    )));

    let process_poll_interval = Duration::from_secs(config.update_interval_seconds.max(1));
    let bash_cfg = config.bash_history.clone();

    {
        let state = state.clone();
        let personality = personality.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move {
                SystemMonitor::monitor_loop(process_poll_interval, move |process, is_new| {
                    if !is_new {
                        return;
                    }
                    let state = state.clone();
                    let personality = personality.clone();
                    tokio::spawn(async move {
                        {
                            let st = state.lock().unwrap();
                            if let Some(last) = st.last_comment_at {
                                if last.elapsed() < st.comment_cooldown {
                                    return; // too soon, skip everything
                                }
                            }
                        }

                        let (comment, animation) =
                            personality.comment_for(&process, CommentKind::Window).await;
                        let mut st = state.lock().unwrap();
                        st.show_comment(comment, animation);
                    });
                })
                .await;
            });
        });
    }

    if bash_cfg.enabled {
        let state = state.clone();
        let personality = personality.clone();
        std::thread::spawn(
            move || match BashHistoryWatcher::new(bash_cfg.history_file.clone()) {
                Some(watcher) => {
                    let rt =
                        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(async move {
                        let poll = Duration::from_secs(bash_cfg.poll_interval_seconds.max(1));
                        let chance = bash_cfg.comment_chance;
                        watcher
                            .watch_loop(poll, chance, move |cmd| {
                                let state = state.clone();
                                let personality = personality.clone();
                                tokio::spawn(async move {
                                    {
                                        let st = state.lock().unwrap();
                                        if let Some(last) = st.last_comment_at {
                                            if last.elapsed() < st.comment_cooldown {
                                                return;
                                            }
                                        }
                                    }

                                    let (comment, animation) = personality
                                        .comment_for(&cmd, CommentKind::BashCommand)
                                        .await;
                                    let mut st = state.lock().unwrap();
                                    st.show_comment(comment, animation);
                                });
                            })
                            .await;
                    });
                }
                None => {
                    eprintln!(
                        "clippy-linux: bash_history is enabled but no history file could be \
                         found (tried $HISTFILE and ~/.bash_history). Set \
                         \"bash_history.history_file\" in config.json to point at it explicitly."
                    );
                }
            },
        );
    }

    wayland::WaylandApp::run(state, assets);
}
