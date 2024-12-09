use super::*;
use crate::map::Map;
use crate::particle_system::ParticleBuilder;

//use specs::prelude::*;

pub fn particle_to_tile(ecs: &mut World, tile_idx: i32, effect: &EffectSpawner) {
    if let EffectType::Particle {
        glyph,
        fg,
        bg,
        lifespan,
    } = effect.effect_type
    {
        let map = ecs.fetch::<Map>();
        let mut particle_builder = ecs.fetch_mut::<ParticleBuilder>();
        particle_builder.request(
            tile_idx % map.width,
            tile_idx / map.width,
            fg,
            bg,
            glyph,
            lifespan,
        );
    }
}

pub fn projectile(ecs: &mut World, tile_idx: i32, effect: &EffectSpawner) {
    if let EffectType::ParticleProjectile {
        glyph,
        fg,
        bg,
        lifespan,
        speed,
        path,
    } = &effect.effect_type
    {
        let map = ecs.fetch::<Map>();
        let (x, y) = map.idx_xy(tile_idx as usize);
        std::mem::drop(map);
        ecs.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                fg: *fg,
                bg: *bg,
                glyph: *glyph,
                render_order: 0,
            })
            .with(ParticleLifetime {
                //lifetime_ms: path.len() as f32 * speed,
                lifetime_ms: *lifespan,
                animation: Some(ParticleAnimation {
                    step_time: *speed,
                    path: path.to_vec(),
                    current_step: 0,
                    timer: 0.0,
                }),
            })
            .build();
    }
}
