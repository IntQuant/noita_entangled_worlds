use image::{Pixel, Rgb};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ffi::OsString, fs, path::PathBuf};

#[derive(Deserialize, Serialize)]
struct Input {
    colors: HashMap<String, [u8; 3]>,
    files: Vec<(String, PathBuf)>,
}

fn main() {
    fs::create_dir_all("masks").unwrap();
    let input = fs::read_to_string("input").unwrap();
    let input = ron::from_str::<Input>(&input).unwrap();
    for (name, file) in input.files {
        let original = image::open(&file).unwrap().into_rgba8();
        for (color_name, color) in &input.colors {
            let mut mask = image::RgbImage::new(original.width(), original.height());
            for (original_pixel, mask_pixel) in original.pixels().zip(mask.pixels_mut()) {
                if original_pixel.channels()[0..3] == *color {
                    *mask_pixel = Rgb([255, 255, 255]);
                }
            }
            let mut mask_name = OsString::new();
            mask_name.push(&name);
            mask_name.push(format!("_{color_name}_mask.png"));
            let mask_path = PathBuf::new().join("masks").join(mask_name);
            mask.save(mask_path).unwrap();
        }
    }
}

#[allow(dead_code)]
fn write_test_input() {
    let content = ron::Options::default()
        .to_string_pretty(&test_input(), ron::ser::PrettyConfig::new())
        .unwrap();
    fs::write("test_input", content).unwrap();
}

#[allow(dead_code)]
fn test_input() -> Input {
    fn s(s: &str) -> String {
        s.to_string()
    }
    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }
    Input {
        colors: HashMap::from_iter([
            (s("main"), [11, 22, 33]),
            (s("alt"), [55, 66, 77]),
            (s("arm"), [99, 100, 111]),
        ]),
        files: vec![
            (s("bowling_balls"), p("bowling/balls")),
            (s("hi_guys"), p("../hi/guys")),
            (s("burnt"), p("i/like/it/burnt")),
        ],
    }
}
