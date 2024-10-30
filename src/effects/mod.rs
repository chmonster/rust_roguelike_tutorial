use crate::map::Map;
use specs::prelude::*;
use std::collections::VecDeque;
use std::sync::Mutex;

mod damage;
mod particles;
pub mod targeting;
pub use targeting::*;

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

pub enum EffectType {
    Damage {
        amount: i32,
    },
    Bloodstain,
    Particle {
        glyph: rltk::FontCharType,
        fg: rltk::RGB,
        bg: rltk::RGB,
        lifespan: f32,
    },
    EntityDeath,
}

#[derive(Clone)]
pub enum Targets {
    Tile { tile_idx: i32 },
    Tiles { tiles: Vec<i32> },
    Single { target: Entity },
    TargetList { targets: Vec<Entity> },
}

pub struct EffectSpawner {
    pub creator: Option<Entity>,
    pub effect_type: EffectType,
    pub targets: Targets,
}

pub fn add_effect(creator: Option<Entity>, effect_type: EffectType, targets: Targets) {
    EFFECT_QUEUE.lock().unwrap().push_back(EffectSpawner {
        creator,
        effect_type,
        targets,
    });
}

pub fn run_effects_queue(ecs: &mut World) {
    loop {
        let effect: Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            target_applicator(ecs, &effect);
        } else {
            break;
        }
    }
}

fn target_applicator(ecs: &mut World, effect: &EffectSpawner) {
    match &effect.targets {
        Targets::Tile { tile_idx } => affect_tile(ecs, effect, *tile_idx),
        Targets::Tiles { tiles } => tiles
            .iter()
            .for_each(|tile_idx| affect_tile(ecs, effect, *tile_idx)),
        Targets::Single { target } => affect_entity(ecs, effect, *target),
        Targets::TargetList { targets } => targets
            .iter()
            .for_each(|entity| affect_entity(ecs, effect, *entity)),
    }
}

fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    matches!(effect, EffectType::Damage { .. })
}

fn affect_tile(ecs: &mut World, effect: &EffectSpawner, tile_idx: i32) {
    if tile_effect_hits_entities(&effect.effect_type) {
        //let content = ecs.fetch::<Map>().tiles[tile_idx as usize].clone();
        let content = crate::spatial::get_tile_content_clone(tile_idx as usize);
        content
            .iter()
            .for_each(|entity| affect_entity(ecs, effect, *entity));

        match &effect.effect_type {
            EffectType::Bloodstain => damage::bloodstain(ecs, tile_idx),
            EffectType::Particle { .. } => particles::particle_to_tile(ecs, tile_idx, effect),
            _ => {}
        }
    }
    // TODO: Run the effect
}

#[allow(unreachable_patterns)]
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage { .. } => damage::inflict_damage(ecs, effect, target),
        EffectType::EntityDeath => damage::death(ecs, effect, target),

        EffectType::Bloodstain { .. } => {
            if let Some(pos) = entity_position(ecs, target) {
                damage::bloodstain(ecs, pos)
            }
        }
        EffectType::Particle { .. } => {
            if let Some(pos) = entity_position(ecs, target) {
                particles::particle_to_tile(ecs, pos, effect)
            }
        }
        _ => {}
    }
}
