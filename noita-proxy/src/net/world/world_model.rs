use std::collections::HashMap;

use image::{Rgb, RgbImage};

use super::RunLengthUpdate;

#[derive(Clone, Copy, Default)]
pub struct Pixel {
    material: i16,
}

pub struct WorldModel {
    pub pixels: HashMap<(i32, i32), Pixel>,
}

impl WorldModel {
    fn set_pixel(&mut self, x: i32, y: i32, pixel: Pixel) {
        self.pixels.insert((x, y), pixel);
    }
    fn get_pixel(&self, x: i32, y: i32) -> Pixel {
        self.pixels.get(&(x, y)).copied().unwrap_or_default()
    }
    pub fn new() -> Self {
        Self {
            pixels: Default::default(),
        }
    }

    pub fn apply_update(&mut self, update: &RunLengthUpdate) {
        let header = &update.header;
        let runs = &update.runs;

        let mut x = 0;
        let mut y = 0;

        for run in runs {
            for _ in 0..(u32::from(run.length) + 1) {
                self.set_pixel(
                    header.x + x,
                    header.y + y,
                    Pixel {
                        material: run.material,
                    },
                );
                x += 1;
                if x == i32::from(header.w) + 1 {
                    x = 0;
                    y += 1;
                }
            }
        }
    }

    pub fn get_start(&self) -> (i32, i32) {
        let x = self.pixels.keys().map(|v| v.0).min().unwrap_or_default();
        let y = self.pixels.keys().map(|v| v.1).min().unwrap_or_default();
        (x, y)
    }

    pub fn gen_image(&self, x: i32, y: i32, w: u32, h: u32) -> RgbImage {
        RgbImage::from_fn(w, h, |lx, ly| {
            let pixel = self.get_pixel(x + lx as i32, y + ly as i32);
            let b = if pixel.material != 0 { 255 } else { 0 };
            Rgb([pixel.material as u8, (pixel.material >> 8) as u8, b])
        })
    }
}
