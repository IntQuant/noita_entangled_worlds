use std::sync::atomic::Ordering;

use image::RgbaImage;
use rustc_hash::FxHashMap;

use eframe::egui::load::TexturePoll;
use eframe::egui::{
    Color32, ColorImage, Context, Key, Rect, Sense, SizeHint, TextureOptions, Ui, Vec2,
    include_image, pos2,
};
use eframe::epaint::TextureHandle;

use shared::WorldPos;
use shared::world_sync::ChunkCoord;

use crate::NetManStopOnDrop;
use crate::net::omni::OmniPeerId;

pub struct ImageMap {
    textures: FxHashMap<ChunkCoord, TextureHandle>,
    zoom: f32,
    offset: Vec2,
    players: FxHashMap<OmniPeerId, (Option<WorldPos>, bool, bool, TextureHandle)>,
    notplayer: Option<TexturePoll>,
    centered_on: Option<OmniPeerId>,
    dont_scale: bool,
}
impl Default for ImageMap {
    fn default() -> Self {
        Self {
            textures: FxHashMap::default(),
            zoom: 1.0,
            offset: Vec2::new(f32::MAX, f32::MAX),
            players: Default::default(),
            notplayer: None,
            centered_on: None,
            dont_scale: false,
        }
    }
}
impl ImageMap {
    pub fn update_textures(
        &mut self,
        ui: &mut Ui,
        map: &FxHashMap<ChunkCoord, RgbaImage>,
        ctx: &Context,
    ) {
        for (coord, img) in map {
            let name = format!("{}x{}", coord.0, coord.1);
            if self.textures.contains_key(coord) {
                ctx.forget_image(&name)
            }
            let size = [img.width() as usize, img.height() as usize];
            let color_image =
                ColorImage::from_rgba_unmultiplied(size, img.as_flat_samples().as_slice());
            let tex = ui
                .ctx()
                .load_texture(name, color_image, TextureOptions::NEAREST);
            self.textures.insert(*coord, tex);
        }
    }

    pub fn update_player_textures(
        &mut self,
        ui: &mut Ui,
        map: &FxHashMap<OmniPeerId, (Option<WorldPos>, bool, bool, RgbaImage)>,
    ) {
        for (p, (coord, is_dead, does_exist, img)) in map {
            if !self.players.contains_key(p) {
                let name = format!("{p}");
                let size = [img.width() as usize, img.height() as usize];
                let color_image =
                    ColorImage::from_rgba_unmultiplied(size, img.as_flat_samples().as_slice());
                let tex = ui
                    .ctx()
                    .load_texture(name, color_image, TextureOptions::NEAREST);
                self.players
                    .insert(*p, (*coord, *is_dead, *does_exist, tex));
            }
            self.players.entry(*p).and_modify(|(w, b, d, _)| {
                *w = *coord;
                *b = *is_dead;
                *d = *does_exist;
            });
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, netman: &NetManStopOnDrop, ctx: &Context) {
        if self.offset == Vec2::new(f32::MAX, f32::MAX) {
            self.offset = Vec2::new(ui.available_width() / 2.0, ui.available_height() / 2.0);
        }
        if netman.reset_map.load(Ordering::Relaxed) {
            netman.reset_map.store(false, Ordering::Relaxed);
            self.textures.clear();
        }
        {
            let map = &mut netman.chunk_map.lock().unwrap();
            if !map.is_empty() {
                self.update_textures(ui, map, ctx);
            }
            map.clear();
        }
        if self.notplayer.is_none() {
            self.notplayer = include_image!("../assets/notplayer.png")
                .load(
                    ctx,
                    TextureOptions::NEAREST,
                    SizeHint::Size {
                        width: 7,
                        height: 17,
                        maintain_aspect_ratio: true,
                    },
                )
                .ok();
        }
        {
            self.update_player_textures(ui, &netman.players_sprite.lock().unwrap());
        }
        let response = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with("map_interact"),
            Sense::drag(),
        );
        if response.dragged() {
            self.offset += response.drag_delta();
        }

        if ui.input(|i| i.raw_scroll_delta.y) != 0.0 {
            let mouse_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
            let mouse_relative = mouse_pos - self.offset;
            let zoom_factor = 2.0_f32.powf(ui.input(|i| i.raw_scroll_delta.y / 256.0));
            self.zoom *= zoom_factor;
            let new_mouse_relative = mouse_relative * zoom_factor;
            self.offset = mouse_pos - new_mouse_relative;
        }
        let s = 32.0;
        if ui.input(|i| i.keys_down.contains(&Key::W) || i.keys_down.contains(&Key::ArrowUp)) {
            self.offset.y += s
        }
        if ui.input(|i| i.keys_down.contains(&Key::S) || i.keys_down.contains(&Key::ArrowDown)) {
            self.offset.y -= s
        }
        if ui.input(|i| i.keys_down.contains(&Key::A) || i.keys_down.contains(&Key::ArrowLeft)) {
            self.offset.x += s
        }
        if ui.input(|i| i.keys_down.contains(&Key::D) || i.keys_down.contains(&Key::ArrowRight)) {
            self.offset.x -= s
        }
        if ui.input(|i| i.key_released(Key::Q)) {
            self.zoom *= 2.0 / 3.0
        }
        if ui.input(|i| i.key_released(Key::E)) {
            self.zoom *= 3.0 / 2.0
        }
        if ui.input(|i| i.key_released(Key::X)) {
            self.dont_scale = !self.dont_scale
        }
        let q = ui.input(|i| i.key_released(Key::Z));
        let e = ui.input(|i| i.key_released(Key::C));
        if q || e {
            let players: Vec<OmniPeerId> = self
                .players
                .iter()
                .filter_map(|(a, (c, _, d, _))| if c.is_some() && !d { Some(a) } else { None })
                .cloned()
                .collect();
            self.centered_on = if !players.is_empty() {
                if let Some(id) = self.centered_on {
                    if let Some(i) = players.iter().position(|o| *o == id) {
                        let i = if q { i as i32 - 1 } else { i as i32 + 1 }
                            .rem_euclid(players.len() as i32 + 1)
                            as usize;
                        if i == players.len() {
                            None
                        } else {
                            Some(players[i])
                        }
                    } else {
                        Some(players[0])
                    }
                } else if q {
                    Some(players[players.len() - 1])
                } else {
                    Some(players[0])
                }
            } else {
                None
            }
        }
        let tile_size = self.zoom * 128.0;
        if let Some(peer) = self.centered_on
            && let Some((Some(pos), _, _, _)) = self.players.get(&peer)
        {
            self.offset = Vec2::new(ui.available_width() / 2.0, ui.available_height() / 2.0)
                - Vec2::new(
                    pos.x as f32 * tile_size / 128.0,
                    (pos.y - 12) as f32 * tile_size / 128.0,
                )
        }
        let painter = ui.painter();
        for (coord, tex) in &self.textures {
            let pos =
                self.offset + Vec2::new(coord.0 as f32 * tile_size, coord.1 as f32 * tile_size);
            let rect = Rect::from_min_size(pos.to_pos2(), Vec2::splat(tile_size));
            painter.image(
                tex.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        for (pos, is_dead, does_exist, tex) in self.players.values() {
            if *does_exist {
                continue;
            }
            if let Some(pos) = pos {
                let pos = self.offset
                    + Vec2::new(
                        pos.x as f32 * tile_size / 128.0,
                        (pos.y - 12) as f32 * tile_size / 128.0,
                    );
                let mut tile_size = tile_size;
                if self.dont_scale && self.zoom < 1.0 {
                    tile_size = 128.0
                }
                let rect = Rect::from_min_size(
                    pos.to_pos2(),
                    Vec2::new(7.0 * tile_size / 128.0, 16.0 * tile_size / 128.0),
                );
                if *is_dead {
                    if let Some(tex) = &self.notplayer
                        && let Some(id) = tex.texture_id()
                    {
                        painter.image(
                            id,
                            rect,
                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                } else {
                    painter.image(
                        tex.id(),
                        rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
}
