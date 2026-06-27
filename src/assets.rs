use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub duration: u32,
    #[serde(default)]
    pub images: Vec<Vec<i32>>,
    #[serde(default)]
    pub sound: Option<String>,

    #[serde(default)]
    pub branching: Option<Value>,
    #[serde(default)]
    pub exit_branch: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Animation {
    pub frames: Vec<Frame>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentData {
    #[allow(dead_code)]
    overlay_count: u32,
    #[allow(dead_code)]
    sounds: Vec<String>,
    #[allow(dead_code)]
    framesize: [u32; 2],
    animations: HashMap<String, Animation>,
}

pub struct AssetManager {
    pub animations: HashMap<String, Animation>,
    pub sprite_sheet: image::DynamicImage,
}

impl AssetManager {
    pub fn load() -> Self {
        Self {
            animations: Self::load_animations(),
            sprite_sheet: Self::load_sprite_sheet(),
        }
    }

    fn load_animations() -> HashMap<String, Animation> {
        const RAW: &str = include_str!("../Clippy/agent.js");

        let start = RAW
            .find('{')
            .expect("Clippy/agent.js: no opening brace found");
        let end = RAW
            .rfind('}')
            .expect("Clippy/agent.js: no closing brace found");
        let json = &RAW[start..=end];

        let data: AgentData = serde_json::from_str(json)
            .expect("Failed to parse embedded Clippy/agent.js animation data");

        let mut animations = data.animations;
        for anim in animations.values_mut() {
            Self::fill_missing_images(anim);
        }
        animations
    }

    fn fill_missing_images(anim: &mut Animation) {
        let mut last_images: Vec<Vec<i32>> = vec![vec![0, 0]];
        for frame in anim.frames.iter_mut() {
            if frame.images.is_empty() {
                frame.images = last_images.clone();
            } else {
                last_images = frame.images.clone();
            }
        }
    }

    fn load_sprite_sheet() -> image::DynamicImage {
        image::load_from_memory(include_bytes!("../Clippy/map_transparent.png"))
            .expect("Failed to load embedded Clippy/map_transparent.png")
    }
}
