use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Frame {
    pub duration: u32,
    pub images: Vec<Vec<i32>>,
    pub sound: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Animation {
    pub frames: Vec<Frame>,
}

pub struct AssetManager {
    pub animations: HashMap<String, Animation>,
    pub sprite_sheet: image::DynamicImage,
}

macro_rules! frame {
    ($dur:expr, $x:expr, $y:expr) => {
        Frame {
            duration: $dur,
            images: vec![vec![$x, $y]],
            sound: None,
        }
    };
}

impl AssetManager {
    pub fn load() -> Self {
        let mut animations = HashMap::new();

        animations.insert(
            "Idle1_1".to_string(),
            Animation {
                frames: vec![
                    frame!(1500, 0, 0),
                    frame!(100, 2108, 744),
                    frame!(100, 2232, 744),
                    frame!(100, 2356, 744),
                    frame!(300, 2480, 744),
                    frame!(100, 2604, 744),
                    frame!(100, 2728, 744),
                    frame!(300, 2852, 744),
                    frame!(100, 2976, 744),
                    frame!(100, 3100, 744),
                    frame!(300, 3224, 744),
                    frame!(100, 0, 837),
                    frame!(100, 124, 837),
                    frame!(300, 248, 837),
                    frame!(100, 372, 837),
                    frame!(100, 496, 837),
                    frame!(300, 620, 837),
                    frame!(100, 744, 837),
                    frame!(100, 868, 837),
                    frame!(300, 992, 837),
                    frame!(100, 1116, 837),
                    frame!(100, 1240, 837),
                    frame!(300, 1364, 837),
                    frame!(100, 1488, 837),
                    frame!(100, 1612, 837),
                    frame!(300, 1736, 837),
                    frame!(100, 1860, 837),
                    frame!(100, 1984, 837),
                    frame!(300, 2108, 837),
                    frame!(100, 2232, 837),
                    frame!(100, 2356, 837),
                    frame!(300, 2480, 837),
                    frame!(100, 2604, 837),
                    frame!(100, 2728, 837),
                    frame!(300, 2852, 837),
                    frame!(100, 2976, 837),
                    frame!(1200, 0, 0),
                ],
            },
        );

        animations.insert(
            "IdleEyeBrowRaise".to_string(),
            Animation {
                frames: vec![
                    frame!(100, 0, 0),
                    frame!(100, 1116, 186),
                    frame!(100, 1240, 186),
                    frame!(900, 1364, 186),
                    frame!(100, 1240, 186),
                    frame!(100, 1116, 186),
                    frame!(100, 0, 0),
                ],
            },
        );

        animations.insert(
            "IdleFingerTap".to_string(),
            Animation {
                frames: vec![
                    frame!(100, 0, 0),
                    frame!(100, 2976, 2976),
                    frame!(100, 3100, 2976),
                    frame!(100, 3224, 2976),
                    frame!(100, 0, 3069),
                    frame!(100, 124, 3069),
                    frame!(150, 248, 3069),
                    frame!(100, 372, 3069),
                    frame!(100, 496, 3069),
                    frame!(100, 620, 3069),
                    frame!(100, 0, 0),
                ],
            },
        );

        animations.insert(
            "LookRight".to_string(),
            Animation {
                frames: vec![
                    frame!(100, 0, 0),
                    frame!(100, 620, 651),
                    frame!(100, 744, 651),
                    frame!(1200, 868, 651),
                    frame!(100, 992, 651),
                    frame!(100, 1116, 651),
                    frame!(100, 0, 0),
                ],
            },
        );

        animations.insert(
            "Thinking".to_string(),
            Animation {
                frames: vec![
                    frame!(100, 0, 0),
                    frame!(100, 124, 93),
                    frame!(100, 248, 93),
                    frame!(100, 372, 93),
                    frame!(100, 496, 93),
                    frame!(100, 620, 93),
                    frame!(100, 744, 93),
                    frame!(100, 868, 93),
                    frame!(100, 992, 93),
                    frame!(100, 1116, 93),
                    frame!(100, 1240, 93),
                    frame!(100, 1364, 93),
                    frame!(100, 1488, 93),
                    frame!(100, 1612, 93),
                    frame!(100, 1736, 93),
                    frame!(100, 1860, 93),
                    frame!(100, 1984, 93),
                    frame!(100, 2108, 93),
                    frame!(100, 2232, 93),
                    frame!(100, 2356, 93),
                    frame!(100, 2480, 93),
                    frame!(100, 2604, 93),
                    frame!(100, 2728, 93),
                    frame!(100, 2852, 93),
                    frame!(100, 2976, 93),
                    frame!(100, 3100, 93),
                    frame!(100, 3224, 93),
                    frame!(100, 0, 186),
                    frame!(100, 124, 186),
                    frame!(100, 248, 186),
                    frame!(100, 372, 186),
                    frame!(100, 496, 186),
                    frame!(100, 620, 186),
                    frame!(100, 744, 186),
                    frame!(100, 868, 186),
                    frame!(100, 992, 186),
                    frame!(100, 0, 0),
                ],
            },
        );

        animations.insert(
            "Alert".to_string(),
            Animation {
                frames: vec![
                    frame!(100, 0, 0),
                    frame!(100, 2356, 1116),
                    frame!(100, 2480, 1116),
                    frame!(100, 2604, 1116),
                    frame!(100, 2728, 1116),
                    frame!(100, 2852, 1116),
                    frame!(100, 2976, 1116),
                    frame!(100, 3100, 1116),
                    frame!(100, 3224, 1116),
                    frame!(100, 0, 1209),
                    frame!(500, 124, 1209),
                    frame!(100, 248, 1209),
                    frame!(100, 372, 1209),
                    frame!(100, 496, 1209),
                    frame!(100, 620, 1209),
                    frame!(100, 744, 1209),
                    frame!(100, 868, 1209),
                    frame!(100, 992, 1209),
                    frame!(100, 1116, 1209),
                    frame!(100, 0, 0),
                ],
            },
        );

        animations.insert(
            "Congratulate".to_string(),
            Animation {
                frames: vec![
                    frame!(100, 0, 0),
                    frame!(50, 124, 0),
                    frame!(50, 248, 0),
                    frame!(50, 372, 0),
                    frame!(50, 496, 0),
                    frame!(50, 620, 0),
                    frame!(50, 744, 0),
                    frame!(50, 868, 0),
                    frame!(50, 992, 0),
                    frame!(100, 1116, 0),
                    frame!(100, 1240, 0),
                    frame!(100, 1364, 0),
                    frame!(1200, 1488, 0),
                    frame!(100, 1612, 0),
                    frame!(100, 1736, 0),
                    frame!(1200, 1488, 0),
                    frame!(100, 1860, 0),
                    frame!(100, 1984, 0),
                    frame!(100, 2108, 0),
                    frame!(100, 2232, 0),
                    frame!(100, 2356, 0),
                    frame!(100, 0, 0),
                ],
            },
        );

        let raw = image::open("Clippy/map.png").expect("Failed to load Clippy/map.png");
        let mut rgba = raw.to_rgba8();
        for pixel in rgba.pixels_mut() {
            if pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 255 {
                *pixel = image::Rgba([0, 0, 0, 0]);
            }
        }
        let sprite_sheet = image::DynamicImage::ImageRgba8(rgba);

        Self {
            animations,
            sprite_sheet,
        }
    }
}
