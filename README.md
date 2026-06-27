# clippy-linux

A Wayland desktop pet that watches what you do and tells you it's wrong.

clippy-linux is a system-wide assistant in the spirit of the original Clippy, except he doesn't want to help you. He wants to judge you. He is a Bitter Old Fucker sysadmin trapped in a paperclip, and he has opinions about your distro, your editor, and your questionable use of `sudo`.

---

## How it works

clippy-linux runs in the corner of your screen and does two things:

- Watches which window is currently focused. When you switch to something worth mocking, he comments on it.
- Tails your bash history. Every command you type is a potential opportunity for him to say something cutting.

Comments are pulled from JSON lists that you can edit, extend, or replace entirely. If you want dynamic insults powered by ai slop including ones that look at a screenshot of your screen that's supported too.

He also knows what distro you're running and adjusts his attitude accordingly. Gentoo users get cold respect. Ubuntu users get fury.

---

## Requirements

- Wayland
- For window tracking: `hyprctl` (Hyprland) or `swaymsg` (Sway). Falls back to the highest CPU process if neither is found.
- Rust toolchain to build

---

## Installation

```sh
git clone https://github.com/JustRoccat/clippy-linux
cd clippy-linux
cargo build --release
./target/release/clippy-linux
```

---

## Bash history

By default, bash only writes history when a shell exits. To get near-realtime reactions, add this to your `~/.bashrc`:

```sh
PROMPT_COMMAND="history -a; $PROMPT_COMMAND"
```

Without it, clippy-linux will still pick up commands eventually just not the moment you type them.

---

## Configuration

Config lives at `~/.config/clippy-linux/config.json` and is created automatically on first run.

```json
{
  "position": {
    "corner": "bottom-right",
    "margin_x": 0,
    "margin_y": 0
  },
  "update_interval_seconds": 5,
  "comment_cooldown_seconds": 40,
  "bubble_show_seconds": 7,
  "bash_history": {
    "enabled": true,
    "history_file": null,
    "comment_chance": 0.25,
    "poll_interval_seconds": 2
  },
  "comment_lists": {
    "directory": "comments",
    "active": []
  },
  "ai-slop": {
    "enabled": false,
    "endpoint": "https://api.openai.com/v1/chat/completions",
    "model": "gpt-4o-mini",
    "api_key": "",
    "system_prompt": "",
    "use_for_window_comments": false,
    "use_for_bash_comments": false,
    "timeout_seconds": 8,
    "screen_vision": false,
    "screen_vision_scope": "window"
  }
}
```

`corner` accepts: `bottom-right`, `bottom-left`, `top-right`, `top-left`.

`comment_chance` is a float between 0 and 1. At `0.25`, roughly one in four bash commands gets a response.

`active` is a list of comment list names to load. If empty, all lists in the directory are loaded.

---

## Comment lists

Comment lists live in `~/.config/clippy-linux/comments/` as JSON files. A set of defaults is written on first run.

Each file looks like this:

```json
{
  "name": "docker",
  "triggers": ["docker"],
  "animation": "Idle1_1",
  "comments": [
    "Congrats on reinventing chroot with more bloat.",
    "Just ship the whole OS, why not."
  ]
}
```

- `triggers`: substrings to match against the window name or command. Case-insensitive. If empty, the list is treated as a general fallback pool.
- `animation`: which Clippy animation to play. Optional.
- `comments`: the actual lines. `%s` is replaced with the matched context string.

You can add as many files as you want. The active comment list is picked randomly from all lists that match the current context.

---

## ai slop integration

Under the `ai-slop` key in config. Supports any OpenAI-compatible endpoint.

When enabled, clippy-linux sends the current context (window name or bash command) to the model and uses the response as the comment. Falls back to static lists if the request times out or fails.

`screen_vision` captures a screenshot via the XDG screenshot portal and includes it in the request. Requires a model with vision support. The screenshot is deleted immediately after being sent.

`system_prompt` overrides the default persona. Leave it empty for the built-in asshole prompt.

---

## Distro hierarchy

clippy-linux reads `/etc/os-release` on startup and adjusts its baseline attitude.

| Distro | Attitude |
|---|---|
| Gentoo / LFS | Cold, grudging respect |
| Arch | Condescending tolerance |
| Debian / Fedora / RHEL | Boredom |
| Everything else | Permanent fury |

---

## License

MIT
