# clippy‑linux documentation

## Architecture

clippy‑linux is split into six modules.

`main.rs` sets up two background threads, one for window monitoring and one for bash history, then hands everything off to the Wayland event loop. Both threads share `AppState` behind a mutex and call into `Personality` to get comment text.

`monitor.rs` handles the two sources of context: active window and bash history. Window detection tries `hyprctl` first, then `swaymsg`, then falls back to grabbing the highest‑CPU process from `ps`. The bash history watcher seeks to the end of the file on startup, then polls for new lines on an interval. It only reports lines that are not empty and do not start with `#` (timestamps from `HISTTIMEFORMAT`).

`personality.rs` owns all the comment logic. On startup it reads JSON lists from the comments directory and builds an index. When asked for a comment, it finds every list whose triggers match the context string, picks one at random, and picks a comment from it at random. If ai slop is enabled, it tries that first and only falls back to static lists on timeout or error.

`llm.rs` is a thin wrapper around an OpenAI‑compatible `/v1/chat/completions` endpoint. It can attach a base64‑encoded screenshot to the user message when `screen_vision` is enabled.

`assets.rs` loads the Clippy sprite sheets and sounds from the embedded `Clippy/` directory.

`wayland.rs` and `ui.rs` handle rendering and the event loop via `smithay‑client‑toolkit`.

---

## Comment list format

Comment lists live in `~/.config/clippy‑linux/comments/` as JSON files. A full example:

```json
{
  "name": "vim",
  "triggers": ["vim", "nvim"],
  "animation": "Thinking",
  "comments": [
    "Vim. The only way to live. Try exiting without a tutorial.",
    ":wq and you still got it wrong, didn't you.",
    "%s  so you can pretend you know what you're doing."
  ]
}
```

### Fields

- **`name`** – required, the list identifier. Used by `comment_lists.active` in config to filter which lists are loaded.
- **`triggers`** – optional, a list of substrings matched case‑insensitively against the current window name or bash command. Any match makes the list a candidate. If triggers is empty, the list becomes a generic fallback and can fire when no trigger‑based list matches.
- **`animation`** – optional, the Clippy animation to play when a comment from this list fires. See the animations section. If omitted, clippy‑linux picks one automatically based on context keywords.
- **`comments`** – required, the list of comment strings. One is picked at random each time the list fires. `%s` anywhere in the string is replaced with the raw context value (window name or command).

---

## Placeholder `%s`

`%s` is a literal placeholder inside any comment string. When Clippy picks a comment, it replaces every occurrence of `%s` with the **context** of the trigger:

- Window / process name – if the comment comes from a window switch  
  (e.g. `firefox`, `kitty`, `nvim`)
- Bash command – if the comment comes from a history watch  
  (e.g. `sudo pacman -Syu`, `git push --force`)

### Example

Comment in a JSON list:
```json
"comments": ["You opened %s. Bold move for someone who still uses a mouse."]
```

If the user opens Firefox, Clippy says:  
*“You opened firefox. Bold move for someone who still uses a mouse.”*

If the user runs `sudo rm -rf /`, Clippy says:  
*“You opened sudo rm -rf /. Bold move for someone who still uses a mouse.”*

The replacement happens in `personality.rs` via a simple `comment.replace("%s", context)` before the string is returned to the UI. The rendered bubble never contains the raw `%s` sequence.

---

## Animations

These are all valid values for the `animation` field.

| Name | Description |
|---|---|
| `Alert` | Clippy looks alarmed. Good for sudo, rm -rf, that kind of thing. |
| `Congratulate` | Clippy begrudgingly acknowledges something. Used for doas. |
| `Thinking` | Clippy pretends to consider something. Good for editors. |
| `Processing` | Similar to Thinking, slightly different pose. |
| `Explain` | Clippy gestures as if explaining something. |
| `Searching` | Clippy looks around. |
| `CheckingSomething` | Clippy peers at something. |
| `Writing` | Clippy writes. Good for text editors, documents. |
| `Print` | Clippy prints something. |
| `Save` | Clippy saves something. |
| `SendMail` | Clippy sends mail. |
| `GetAttention` | Clippy tries to get your attention. |
| `GetTechy` | Clippy goes into tech mode. |
| `GetWizardy` | Clippy does something arcane. |
| `GetArtsy` | Clippy gets creative. |
| `GoodBye` | Clippy waves goodbye. |
| `Greeting` | Clippy says hello. |
| `Wave` | Clippy waves. |
| `Hide` | Clippy disappears. |
| `Show` | Clippy appears. |
| `EmptyTrash` | Clippy empties something. |
| `IdleAtom` | Idle animation with an atom orbiting. |
| `IdleEyeBrowRaise` | Clippy raises an eyebrow at you. |
| `IdleFingerTap` | Clippy taps impatiently. |
| `IdleHeadScratch` | Clippy scratches his head. |
| `IdleRopePile` | Clippy does something with a rope. |
| `IdleSideToSide` | Clippy sways. |
| `IdleSnooze` | Clippy falls asleep. Good for long‑running commands. |
| `Idle1_1` | Default idle. This is the fallback when nothing else matches. |
| `RestPose` | Clippy sits still. |
| `Hearing_1` | Clippy listens. |
| `LookDown` | Clippy looks down. |
| `LookDownLeft` | Clippy looks down and to the left. |
| `LookDownRight` | Clippy looks down and to the right. |
| `LookLeft` | Clippy looks left. |
| `LookRight` | Clippy looks right. |
| `LookUp` | Clippy looks up. |
| `LookUpLeft` | Clippy looks up and to the left. |
| `LookUpRight` | Clippy looks up and to the right. |
| `GestureDown` | Clippy gestures downward. |
| `GestureLeft` | Clippy gestures left. |
| `GestureRight` | Clippy gestures right. |
| `GestureUp` | Clippy gestures upward. |

If you omit the `animation` field entirely, clippy‑linux picks one automatically based on keywords in the context string: `Alert` for sudo, `Thinking` for vim/nvim/emacs, `Congratulate` for doas, `Idle1_1` for everything else.

---

## Distro detection

On startup, `personality.rs` reads `/etc/os‑release` and assigns one of four classes:

- `Elite` — Gentoo or Linux From Scratch
- `Middle` — Arch Linux
- `Worker` — Debian, Fedora, Red Hat
- `Bottom` — anything else

This class affects two things: the default comments written into `distro.json` on first run, and the flavor text prepended to the ai slop context when dynamic comments are enabled.

---

## AI slop

The default system prompt sent to the model:

> You are Clippy, a toxic, elitist BOFH trapped in a Linux desktop pet. Your job is to insult the user in a short, cutting, and painfully accurate way. Never be helpful, never explain, never predict sports. Only roast.

The user message describes what the user is currently doing and asks for one snarky sentence. The distro class is included in the context so the model can adjust its tone accordingly.

You can replace the system prompt entirely via `ai‑slop.system_prompt` in config. If the field is an empty string, the default above is used.

### Screen vision

When `screen_vision` is true, clippy‑linux captures a screenshot via the XDG screenshot portal, encodes it as base64 PNG, and attaches it to the user message. The file is deleted immediately after being read. Requires a model with vision support (gpt‑4o works, gpt‑4o‑mini works, most local models do not).

Detailed steps:

1. Captures a screenshot via the XDG Desktop Portal.
2. The portal returns a `file://` URI, usually pointing to a temporary PNG in `/tmp`.
3. Clippy reads the entire file into memory, **immediately deletes the temporary file**, and encodes the image as base64.
4. The base64 string is attached to the API request as a `data:image/png;base64,...` URL inside the `image_url` content part.

The temporary file is never kept – the line `std::fs::remove_file(path)` runs right after reading, so no screenshots accumulate on disk.

### Vision model requirements

The endpoint must support the OpenAI vision format (the `image_url` key with a base64 data URI). Models known to work:

- `gpt-4o`, `gpt‑4‑turbo`, `gpt‑4o‑mini`
- Any OpenAI‑compatible endpoint with a multimodal model (e.g. some Llava variants)

Most pure text models (including `Phi‑4` non‑multimodal versions) will ignore or reject the image. If you enable `screen_vision` with such a model, the request will either fail or the model will behave as if the image wasn’t there.

### Timeout and fallback

If the AI request times out (`ai‑slop.timeout_seconds`) or returns an error, Clippy discards the screenshot and falls back to the static comment lists without any user‑visible interruption. The screenshot capture itself can also fail silently if the portal is not running or refuses the request – in that case a warning is printed to stderr and the AI call proceeds without an image.

---

## Bash history realtime setup

bash only flushes history to disk when a shell exits, which means clippy‑linux will not react to most commands in a running session unless you tell bash to flush immediately. Add to `~/.bashrc`:

```sh
PROMPT_COMMAND="history -a; $PROMPT_COMMAND"
```

`history -a` appends new commands to the history file after every prompt. clippy‑linux polls the file every `bash_history.poll_interval_seconds` seconds and picks up anything new since the last check.

`bash_history.comment_chance` is a float between 0 and 1 controlling the probability that any given new command triggers a comment. At the default of `0.25`, roughly one in four commands gets a response. Set it to `1.0` for a comment on everything, or `0.0` to disable bash reactions without touching the `enabled` flag.

If your history file is in a non‑standard location (zsh users, for example), set `bash_history.history_file` to the full path. If left null, clippy‑linux checks `$HISTFILE` and then `~/.bash_history`.

---

## Building for release

The release profile in `Cargo.toml` is already configured:

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"
```

```sh
cargo build --release
```

The binary ends up at `target/release/clippy-linux`. It has no runtime dependencies beyond the system Wayland libraries.

---

## Adding and managing comment lists

Drop a JSON file into `~/.config/clippy-linux/comments/`. It will be loaded on next launch. The lists are read once at startup, so changes to existing files also require a restart.

To load only specific lists, set `comment_lists.active` in config to an array of list names:

```json
"comment_lists": {
  "active": ["general", "vim", "sudo"]
}
```

Any list whose `name` is not in the array is ignored. An empty array loads everything.

To use a different directory entirely, set `comment_lists.directory` to an absolute path:

```json
"comment_lists": {
  "directory": "/home/you/my-clippy-comments"
}
```

Relative paths are resolved against the config directory (`~/.config/clippy-linux/`).
