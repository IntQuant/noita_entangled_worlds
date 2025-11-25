use std::collections::HashMap;

use noita_api::{
    noita::types::{GridWorld, Vec2i},
    raw::game_create_sprite_for_x_frames,
};

pub(crate) struct ChangeTracker {
    changed: HashMap<Vec2i, usize>, // Could be just an array of bools probably
}

impl ChangeTracker {
    const CHUNKLET_SIZE_POWER: isize = 6;
    const CHUNKLET_SIZE: isize = 1 << Self::CHUNKLET_SIZE_POWER;

    pub fn new() -> Self {
        Self {
            changed: HashMap::new(),
        }
    }

    pub fn update(&mut self, grid_world: &GridWorld) {
        let grid_world_thread_impl = &grid_world.m_thread_impl;

        // let updated_regions = &grid_world_thread_impl.updated_grid_worlds.as_ref()
        //     [..grid_world.m_thread_impl.chunk_update_count];

        // for updated in updated_regions {
        //     game_create_sprite_for_x_frames(
        //         "mods/quant.ew/files/resource/debug/marker.png".into(),
        //         updated.update_region.top_left.x as f64,
        //         updated.update_region.top_left.y as f64,
        //         Some(true),
        //         Some(0.0),
        //         Some(0.0),
        //         Some(2),
        //         Some(true),
        //     )
        //     .unwrap();
        // }

        let updated_regions = &grid_world_thread_impl.world_update_params.as_ref()
            [..grid_world.m_thread_impl.world_update_params_count];

        for updated in updated_regions {
            let tl = &updated.update_region.top_left;
            let br = &updated.update_region.bottom_right;
            let area = usize::try_from((br.x - tl.x) * (br.y - tl.y)).unwrap();

            let tl_chunklet = Vec2i {
                x: tl.x >> Self::CHUNKLET_SIZE_POWER,
                y: tl.y >> Self::CHUNKLET_SIZE_POWER,
            };
            let br_chunklet = Vec2i {
                x: (br.x + Self::CHUNKLET_SIZE - 1) >> Self::CHUNKLET_SIZE_POWER,
                y: (br.y + Self::CHUNKLET_SIZE - 1) >> Self::CHUNKLET_SIZE_POWER,
            };
            for x in tl_chunklet.x..br_chunklet.x {
                for y in tl_chunklet.y..br_chunklet.y {
                    self.changed
                        .entry(Vec2i { x, y })
                        .and_modify(|x| *x = x.saturating_add(area))
                        .or_insert(area);
                }
            }
        }

        let limit = 2048;
        for (changed, amount) in &self.changed {
            if *amount >= limit {
                // print!("amount: {}", amount);
                game_create_sprite_for_x_frames(
                    "mods/quant.ew/files/resource/debug/marker.png".into(),
                    (changed.x << Self::CHUNKLET_SIZE_POWER) as f64,
                    (changed.y << Self::CHUNKLET_SIZE_POWER) as f64,
                    Some(false),
                    Some(0.0),
                    Some(0.0),
                    Some(2),
                    Some(true),
                )
                .unwrap();
            }
        }
        self.changed.retain(|_k, v| *v < limit);
        // self.changed.clear();
    }
}
