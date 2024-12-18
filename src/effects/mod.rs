use super::{ParticleAnimation, ParticleLifetime, Position, Renderable};
use crate::{map::Map, AttributeBonus};
use rltk::{console, Point};
use specs::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;

mod damage;
mod movement;
mod particles;
pub mod targeting;
mod triggers;
pub use targeting::*;
mod hunger;

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

#[derive(Debug)]
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
    ParticleProjectile {
        glyph: rltk::FontCharType,
        fg: rltk::RGB,
        bg: rltk::RGB,
        lifespan: f32,
        speed: f32,
        path: Vec<Point>,
    },

    EntityDeath,
    ItemUse {
        item: Entity,
    },
    SpellUse {
        spell: Entity,
    },

    WellFed,
    Healing {
        amount: i32,
    },
    Mana {
        amount: i32,
    },
    XP {
        amount: i32,
    },
    Slow {
        initiative_penalty: f32,
    },
    DamageOverTime {
        damage: i32,
    },
    Confusion {
        turns: i32,
    },
    TriggerFire {
        trigger: Entity,
    },
    TeleportTo {
        x: i32,
        y: i32,
        depth: i32,
        player_only: bool,
    },
    AttributeEffect {
        bonus: AttributeBonus,
        name: String,
        duration: i32,
    },
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Targets {
    Tile { tile_idx: i32 },
    Tiles { tiles: Vec<i32> },
    Single { target: Entity },
    TargetList { targets: Vec<Entity> },
}

#[derive(Debug)]
pub struct EffectSpawner {
    pub creator: Option<Entity>,
    pub effect_type: EffectType,
    pub targets: Targets,
    dedupe: HashSet<Entity>,
}

pub fn add_effect(creator: Option<Entity>, effect_type: EffectType, targets: Targets) {
    EFFECT_QUEUE.lock().unwrap().push_back(EffectSpawner {
        creator,
        effect_type,
        targets,
        dedupe: HashSet::new(),
    });
}

pub fn run_effects_queue(ecs: &mut World) {
    let mut i = 0;
    loop {
        let effect: Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(mut effect) = effect {
            i += 1;
            console::log(format!("effect fires {}", i));
            target_applicator(ecs, &mut effect);
        } else {
            break;
        }
    }
}

fn target_applicator(ecs: &mut World, effect: &mut EffectSpawner) {
    if let EffectType::ItemUse { item } = effect.effect_type {
        triggers::item_trigger(effect.creator, item, &effect.targets, ecs);
    } else if let EffectType::SpellUse { spell } = effect.effect_type {
        triggers::spell_trigger(effect.creator, spell, &effect.targets, ecs);
    } else if let EffectType::TriggerFire { trigger } = effect.effect_type {
        triggers::prop_trigger(effect.creator, trigger, &effect.targets, ecs);
    } else {
        match &effect.targets.clone() {
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
}

fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    matches!(
        effect,
        EffectType::Damage { .. }
            | EffectType::WellFed
            | EffectType::Confusion { .. }
            | EffectType::Healing { .. }
            | EffectType::Mana { .. }
            | EffectType::XP { .. }
            | EffectType::TeleportTo { .. }
            | EffectType::AttributeEffect { .. }
            | EffectType::Slow { .. }
            | EffectType::DamageOverTime { .. }
    )
}

fn affect_tile(ecs: &mut World, effect: &mut EffectSpawner, tile_idx: i32) {
    if tile_effect_hits_entities(&effect.effect_type) {
        let content = crate::spatial::get_tile_content_clone(tile_idx as usize);
        content
            .iter()
            .for_each(|entity| affect_entity(ecs, effect, *entity));

        match &effect.effect_type {
            EffectType::Bloodstain => damage::bloodstain(ecs, tile_idx),
            EffectType::Particle { .. } => particles::particle_to_tile(ecs, tile_idx, effect),
            EffectType::ParticleProjectile { .. } => particles::projectile(ecs, tile_idx, effect),

            _ => {}
        }
    }
}

#[allow(unreachable_patterns)]
fn affect_entity(ecs: &mut World, effect: &mut EffectSpawner, target: Entity) {
    if effect.dedupe.contains(&target) {
        return;
    }
    effect.dedupe.insert(target);
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
        EffectType::WellFed => hunger::well_fed(ecs, effect, target),
        EffectType::Healing { .. } => damage::heal_damage(ecs, effect, target),
        EffectType::Mana { .. } => damage::restore_mana(ecs, effect, target),
        EffectType::XP { .. } => damage::give_xp(ecs, effect, target),

        EffectType::Confusion { .. } => damage::add_confusion(ecs, effect, target),
        EffectType::TeleportTo { .. } => movement::apply_teleport(ecs, effect, target),
        EffectType::AttributeEffect { .. } => damage::attribute_effect(ecs, effect, target),
        EffectType::Slow { .. } => damage::slow(ecs, effect, target),
        EffectType::DamageOverTime { .. } => damage::damage_over_time(ecs, effect, target),

        _ => {}
    }
}
