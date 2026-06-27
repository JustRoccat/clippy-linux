#![allow(dead_code)]

use crate::assets::AssetManager;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tiny_skia::*;

pub const CLIPPY_W: u32 = 124;
pub const CLIPPY_H: u32 = 93;

pub const CLIPPY_DRAW_W: u32 = CLIPPY_W;
pub const CLIPPY_DRAW_H: u32 = CLIPPY_H;

pub const WINDOW_W: i32 = 280;
pub const WINDOW_H: i32 = 200;

#[derive(Debug, Clone, PartialEq)]
pub enum AnimState {
    Idle,
    EyeBrowRaise,
    FingerTap,
    LookRight,
    Thinking,
    Alert,
    Congratulate,
}

impl AnimState {
    pub fn animation_name(&self) -> &str {
        match self {
            AnimState::Idle => "Idle1_1",
            AnimState::EyeBrowRaise => "IdleEyeBrowRaise",
            AnimState::FingerTap => "IdleFingerTap",
            AnimState::LookRight => "LookRight",
            AnimState::Thinking => "Thinking",
            AnimState::Alert => "Alert",
            AnimState::Congratulate => "Congratulate",
        }
    }

    pub fn from_name(name: &str) -> AnimState {
        match name {
            "IdleEyeBrowRaise" => AnimState::EyeBrowRaise,
            "IdleFingerTap" => AnimState::FingerTap,
            "LookRight" => AnimState::LookRight,
            "Thinking" => AnimState::Thinking,
            "Alert" => AnimState::Alert,
            "Congratulate" => AnimState::Congratulate,
            _ => AnimState::Idle,
        }
    }

    fn loops(&self) -> bool {
        matches!(self, AnimState::Idle)
    }

    fn idle_quirks() -> Vec<AnimState> {
        vec![
            AnimState::EyeBrowRaise,
            AnimState::FingerTap,
            AnimState::LookRight,
        ]
    }
}

pub struct AppState {
    pub current_comment: String,
    pub current_animation: String,
    pub anim_state: AnimState,
    pub frame_index: usize,
    pub last_frame_update: Instant,
    pub last_anim_change: Instant,
    pub bubble_shown_at: Option<Instant>,
    pub last_comment_at: Option<Instant>,
    pub comment_cooldown: Duration,
    pub bubble_show: Duration,
    pub next_idle_switch_after: u64,
    pub last_idle_variant: Option<AnimState>,
    pub dirty: bool,
}

impl AppState {
    pub fn new(comment_cooldown: Duration, bubble_show: Duration) -> Self {
        let now = Instant::now();
        Self {
            current_comment: String::new(),
            current_animation: "Idle1_1".to_string(),
            anim_state: AnimState::Idle,
            frame_index: 0,
            last_frame_update: now,
            last_anim_change: now,
            bubble_shown_at: None,
            last_comment_at: None,
            comment_cooldown,
            bubble_show,
            next_idle_switch_after: Self::random_idle_interval(),
            last_idle_variant: None,
            dirty: true,
        }
    }

    fn random_idle_interval() -> u64 {
        use rand::Rng;
        rand::thread_rng().gen_range(10..=22)
    }

    fn validate_animation_name(name: &str) -> String {
        match name {
            "Idle1_1" | "IdleEyeBrowRaise" | "IdleFingerTap" | "LookRight" | "Thinking"
            | "Alert" | "Congratulate" => name.to_string(),
            _ => "Idle1_1".to_string(),
        }
    }

    pub fn show_comment(&mut self, comment: String, animation: String) {
        let now = Instant::now();
        if let Some(last) = self.last_comment_at {
            if now.duration_since(last) < self.comment_cooldown {
                return;
            }
        }
        let animation = Self::validate_animation_name(&animation);
        self.current_comment = comment;
        self.current_animation = animation;
        self.anim_state = AnimState::from_name(&self.current_animation);
        self.frame_index = 0;
        self.last_frame_update = now;
        self.bubble_shown_at = Some(now);
        self.last_comment_at = Some(now);
        self.dirty = true;
    }
}

pub struct UiRenderer {
    pub state: Arc<Mutex<AppState>>,
    pub assets: Arc<AssetManager>,
}

impl UiRenderer {
    pub fn new(state: Arc<Mutex<AppState>>, assets: Arc<AssetManager>) -> Self {
        Self { state, assets }
    }

    pub fn render(&self, pixmap: &mut Pixmap) {
        pixmap.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 0));

        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        if let Some(shown_at) = state.bubble_shown_at {
            if now.duration_since(shown_at) >= state.bubble_show {
                state.current_comment = String::new();
                state.bubble_shown_at = None;
                state.current_animation = "Idle1_1".to_string();
                state.anim_state = AnimState::Idle;
                state.frame_index = 0;
                state.last_frame_update = now;
                state.last_anim_change = now;
                state.next_idle_switch_after = AppState::random_idle_interval();
                state.dirty = true;
            }
        }

        if let Some(anim) = self.assets.animations.get(&state.current_animation) {
            if !anim.frames.is_empty() {
                let last_idx = anim.frames.len() - 1;
                let idx = state.frame_index.min(last_idx);
                let frame_duration = anim.frames[idx].duration as u128;
                if now.duration_since(state.last_frame_update).as_millis() >= frame_duration {
                    if idx < last_idx {
                        state.frame_index = idx + 1;
                        state.last_frame_update = now;
                        state.dirty = true;
                    } else if state.anim_state.loops() {
                        state.frame_index = 0;
                        state.last_frame_update = now;
                        state.dirty = true;
                    }
                }
            }
        }

        let anim_finished = self
            .assets
            .animations
            .get(&state.current_animation)
            .map(|a| !a.frames.is_empty() && state.frame_index >= a.frames.len() - 1)
            .unwrap_or(true);

        let is_idle_quirk = matches!(
            state.anim_state,
            AnimState::EyeBrowRaise | AnimState::FingerTap | AnimState::LookRight
        );
        if is_idle_quirk && anim_finished && state.current_comment.is_empty() {
            state.anim_state = AnimState::Idle;
            state.current_animation = "Idle1_1".to_string();
            state.frame_index = 0;
            state.last_frame_update = now;
            state.last_anim_change = now;
            state.dirty = true;
        }

        let elapsed_since_change = now.duration_since(state.last_anim_change).as_secs();
        if state.anim_state == AnimState::Idle
            && state.current_comment.is_empty()
            && elapsed_since_change >= state.next_idle_switch_after
        {
            use rand::Rng;
            let mut variants = AnimState::idle_quirks();
            if let Some(last) = state.last_idle_variant.clone() {
                if variants.len() > 1 {
                    variants.retain(|v| *v != last);
                }
            }
            let pick = rand::thread_rng().gen_range(0..variants.len());
            let new_state = variants[pick].clone();

            state.last_idle_variant = Some(new_state.clone());
            state.anim_state = new_state;
            state.current_animation = state.anim_state.animation_name().to_string();
            state.frame_index = 0;
            state.last_frame_update = now;
            state.last_anim_change = now;
            state.next_idle_switch_after = AppState::random_idle_interval();
            state.dirty = true;
        }

        let show_bubble = !state.current_comment.is_empty();
        let comment = state.current_comment.clone();
        drop(state);

        if show_bubble {
            self.draw_speech_bubble(pixmap, &comment);
        }

        let state = self.state.lock().unwrap();

        if let Some(anim) = self.assets.animations.get(&state.current_animation) {
            if let Some(frame) = anim.frames.get(state.frame_index % anim.frames.len()) {
                if let Some(coords) = frame.images.first() {
                    let sx = coords[0].max(0) as u32;
                    let sy = coords[1].max(0) as u32;

                    let sub_img = self
                        .assets
                        .sprite_sheet
                        .crop_imm(sx, sy, CLIPPY_W, CLIPPY_H);
                    let sub_rgba = sub_img.to_rgba8();

                    if let Some(mut pb) = Pixmap::new(CLIPPY_DRAW_W, CLIPPY_DRAW_H) {
                        if pb.data_mut().len() == sub_rgba.len() {
                            pb.data_mut().copy_from_slice(&sub_rgba);
                            let dx = (WINDOW_W as u32).saturating_sub(CLIPPY_DRAW_W + 8);
                            let dy = (WINDOW_H as u32).saturating_sub(CLIPPY_DRAW_H + 8);
                            pixmap.draw_pixmap(
                                dx as i32,
                                dy as i32,
                                pb.as_ref(),
                                &PixmapPaint::default(),
                                Transform::identity(),
                                None,
                            );
                        }
                    }
                }
            }
        }
    }

    fn draw_speech_bubble(&self, pixmap: &mut Pixmap, text: &str) {
        let padding_h: f32 = 16.0;
        let padding_v: f32 = 12.0;
        let line_height: f32 = 16.0;
        let char_w: f32 = 7.0;
        let max_chars: usize = 26;
        let corner_r: f32 = 12.0;

        let lines = self.wrap_text(text, max_chars);
        let text_h = lines.len() as f32 * line_height;
        let max_line_px = lines
            .iter()
            .map(|l| l.len() as f32 * char_w)
            .fold(0.0_f32, f32::max);
        let bubble_w = (max_line_px + padding_h * 2.0)
            .max(80.0)
            .min(WINDOW_W as f32 - 16.0);
        let bubble_h = text_h + padding_v * 2.0;

        let clippy_top = (WINDOW_H as f32) - (CLIPPY_DRAW_H as f32) - 8.0;
        let clippy_right = (WINDOW_W as f32) - 8.0;
        let bx = (clippy_right - bubble_w).max(8.0);
        let by = (clippy_top - bubble_h - 18.0).max(8.0);

        self.fill_rounded_rect(
            pixmap,
            bx + 3.0,
            by + 3.0,
            bubble_w,
            bubble_h,
            corner_r,
            tiny_skia::Color::from_rgba8(0, 0, 0, 40),
        );

        self.fill_rounded_rect(
            pixmap,
            bx,
            by,
            bubble_w,
            bubble_h,
            corner_r,
            tiny_skia::Color::from_rgba8(255, 252, 230, 248),
        );

        self.stroke_rounded_rect(
            pixmap,
            bx,
            by,
            bubble_w,
            bubble_h,
            corner_r,
            tiny_skia::Color::from_rgba8(30, 50, 120, 230),
            2.0,
        );

        let tail_x = bx + bubble_w - 28.0;
        let tail_top = by + bubble_h;
        let tail_tip_x = tail_x + 10.0;
        let tail_tip_y = tail_top + 17.0;

        let mut tb = PathBuilder::new();
        tb.move_to(tail_x, tail_top);
        tb.line_to(tail_tip_x, tail_tip_y);
        tb.line_to(tail_x + 22.0, tail_top);
        tb.close();
        if let Some(tp) = tb.finish() {
            let mut fill = Paint::default();
            fill.set_color(tiny_skia::Color::from_rgba8(255, 252, 230, 248));
            pixmap.fill_path(&tp, &fill, FillRule::Winding, Transform::identity(), None);

            let mut border = Paint::default();
            border.set_color(tiny_skia::Color::from_rgba8(30, 50, 120, 230));
            let stroke = Stroke {
                width: 2.0,
                ..Default::default()
            };
            pixmap.stroke_path(&tp, &border, &stroke, Transform::identity(), None);

            let mut cover = Paint::default();
            cover.set_color(tiny_skia::Color::from_rgba8(255, 252, 230, 248));
            if let Some(cover_rect) = Rect::from_xywh(tail_x + 1.0, tail_top - 1.5, 22.0, 4.0) {
                let cp = PathBuilder::from_rect(cover_rect);
                pixmap.fill_path(&cp, &cover, FillRule::Winding, Transform::identity(), None);
            }
        }

        let mut paint = Paint::default();
        paint.set_color(tiny_skia::Color::from_rgba8(20, 20, 60, 245));
        paint.anti_alias = true;

        for (i, line) in lines.iter().enumerate() {
            let x = bx + padding_h;
            let y = by + padding_v + (i as f32) * line_height + 1.0;

            for (ci, ch) in line.chars().enumerate() {
                let cx = x + (ci as f32) * char_w;
                if let Some(glyph) = Self::simple_glyph(ch) {
                    for &(dx, dy) in &glyph {
                        if let Some(r) = Rect::from_xywh(cx + dx, y + dy, 1.3, 1.3) {
                            let p = PathBuilder::from_rect(r);
                            pixmap.fill_path(
                                &p,
                                &paint,
                                FillRule::Winding,
                                Transform::identity(),
                                None,
                            );
                        }
                    }
                }
            }
        }
    }

    fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<Path> {
        let r = r.min(w / 2.0).min(h / 2.0);
        let mut pb = PathBuilder::new();
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.quad_to(x + w, y, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.quad_to(x + w, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.quad_to(x, y + h, x, y + h - r);
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);
        pb.close();
        pb.finish()
    }

    fn fill_rounded_rect(
        &self,
        pixmap: &mut Pixmap,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        color: tiny_skia::Color,
    ) {
        if let Some(path) = Self::rounded_rect_path(x, y, w, h, r) {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            pixmap.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    fn stroke_rounded_rect(
        &self,
        pixmap: &mut Pixmap,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        color: tiny_skia::Color,
        width: f32,
    ) {
        if let Some(path) = Self::rounded_rect_path(x, y, w, h, r) {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            let stroke = Stroke {
                width,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    fn wrap_text(&self, text: &str, max_chars: usize) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut current = String::new();

        for word in words {
            if current.len() + word.len() + 1 > max_chars && !current.is_empty() {
                lines.push(current.clone());
                current.clear();
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
        if !current.is_empty() {
            lines.push(current);
        }
        if lines.is_empty() {
            lines.push(text.to_string());
        }
        lines
    }

    fn simple_glyph(ch: char) -> Option<Vec<(f32, f32)>> {
        let bits: &[u8] = match ch {
            'A' => &[
                0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
            'B' => &[
                0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
            ],
            'C' => &[
                0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
            ],
            'D' => &[
                0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
            ],
            'E' => &[
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
            ],
            'F' => &[
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
            'G' => &[
                0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
            ],
            'H' => &[
                0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
            'I' => &[
                0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
            'J' => &[
                0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
            ],
            'K' => &[
                0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
            ],
            'L' => &[
                0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
            ],
            'M' => &[
                0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
            ],
            'N' => &[
                0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001,
            ],
            'O' => &[
                0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            'P' => &[
                0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
            'Q' => &[
                0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
            ],
            'R' => &[
                0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
            ],
            'S' => &[
                0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110,
            ],
            'T' => &[
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
            'U' => &[
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            'V' => &[
                0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
            ],
            'W' => &[
                0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
            ],
            'X' => &[
                0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
            ],
            'Y' => &[
                0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
            'Z' => &[
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
            ],
            'a' => &[
                0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111,
            ],
            'b' => &[
                0b10000, 0b10000, 0b10110, 0b10001, 0b10001, 0b10001, 0b11110,
            ],
            'c' => &[
                0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10001, 0b01110,
            ],
            'd' => &[
                0b00001, 0b00001, 0b01101, 0b10001, 0b10001, 0b10001, 0b01111,
            ],
            'e' => &[
                0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110,
            ],
            'f' => &[
                0b00110, 0b01001, 0b01000, 0b11100, 0b01000, 0b01000, 0b01000,
            ],
            'g' => &[
                0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110,
            ],
            'h' => &[
                0b10000, 0b10000, 0b10110, 0b10001, 0b10001, 0b10001, 0b10001,
            ],
            'i' => &[
                0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
            'j' => &[
                0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100,
            ],
            'k' => &[
                0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010,
            ],
            'l' => &[
                0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
            'm' => &[
                0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10101, 0b10001,
            ],
            'n' => &[
                0b00000, 0b00000, 0b10110, 0b10001, 0b10001, 0b10001, 0b10001,
            ],
            'o' => &[
                0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            'p' => &[
                0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000,
            ],
            'q' => &[
                0b00000, 0b00000, 0b01101, 0b10001, 0b01111, 0b00001, 0b00001,
            ],
            'r' => &[
                0b00000, 0b00000, 0b10110, 0b10001, 0b10000, 0b10000, 0b10000,
            ],
            's' => &[
                0b00000, 0b00000, 0b01110, 0b10000, 0b01110, 0b00001, 0b11110,
            ],
            't' => &[
                0b01000, 0b01000, 0b11100, 0b01000, 0b01000, 0b01001, 0b00110,
            ],
            'u' => &[
                0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101,
            ],
            'v' => &[
                0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
            ],
            'w' => &[
                0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010,
            ],
            'x' => &[
                0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001,
            ],
            'y' => &[
                0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110,
            ],
            'z' => &[
                0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111,
            ],
            '0' => &[
                0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
            ],
            '1' => &[
                0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
            '2' => &[
                0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
            ],
            '3' => &[
                0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110,
            ],
            '4' => &[
                0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
            ],
            '5' => &[
                0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
            ],
            '6' => &[
                0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
            ],
            '7' => &[
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
            ],
            '8' => &[
                0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
            ],
            '9' => &[
                0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
            ],
            '.' => &[
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
            ],
            ',' => &[
                0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b00100, 0b01000,
            ],
            '!' => &[
                0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
            ],
            '?' => &[
                0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
            ],
            ':' => &[
                0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000,
            ],
            ';' => &[
                0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b00100, 0b01000,
            ],
            ' ' => &[
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
            '-' => &[
                0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
            ],
            '\'' => &[
                0b00100, 0b00100, 0b01000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
            '"' => &[
                0b01010, 0b01010, 0b10100, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
            '/' => &[
                0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
            ],
            _ => return None,
        };

        let mut pixels = Vec::new();
        for (row, &byte) in bits.iter().enumerate() {
            for col in 0..5u8 {
                if byte & (1 << (4 - col)) != 0 {
                    pixels.push((col as f32, row as f32));
                }
            }
        }
        Some(pixels)
    }
}
