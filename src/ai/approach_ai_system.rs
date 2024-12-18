use crate::{ApplyMove, /*EntityMoved,*/ Map, MyTurn, Position, /*Viewshed,*/ WantsToApproach,};
use specs::prelude::*;

pub struct ApproachAI {}

impl<'a> System<'a> for ApproachAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        //WriteStorage<'a, Viewshed>,
        //WriteStorage<'a, EntityMoved>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut turns,
            mut want_approach,
            mut positions,
            map,
            //mut viewsheds,
            //mut entity_moved,
            entities,
            mut apply_move,
        ) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, pos, approach, /*viewshed,*/ _myturn) in (
            &entities,
            &mut positions,
            &want_approach,
            //&mut viewsheds,
            &turns,
        )
            .join()
        {
            turn_done.push(entity);
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(approach.idx % map.width, approach.idx / map.width) as i32,
                &*map,
            );
            if path.success && path.steps.len() > 1 {
                /*let idx = map.xy_idx(pos.x, pos.y);
                pos.x = path.steps[1] as i32 % map.width;
                pos.y = path.steps[1] as i32 / map.width;
                entity_moved
                    .insert(entity, EntityMoved {})
                    .expect("Unable to insert marker");
                let new_idx = map.xy_idx(pos.x, pos.y);
                crate::spatial::move_entity(entity, idx, new_idx);
                viewshed.dirty = true;*/
                apply_move
                    .insert(
                        entity,
                        ApplyMove {
                            dest_idx: path.steps[1],
                        },
                    )
                    .expect("Unable to insert");
            }
        }

        want_approach.clear();

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
