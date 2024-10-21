use crate::{Confusion, GameLog, MyTurn, RunState};
use specs::prelude::*;
pub struct TurnStatusSystem {}

impl<'a> System<'a> for TurnStatusSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, Confusion>,
        Entities<'a>,
        ReadExpect<'a, RunState>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut confusion, entities, runstate, mut gamelog) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        let mut not_my_turn: Vec<Entity> = Vec::new();
        let mut not_confused: Vec<Entity> = Vec::new();
        let mut conf_turns: i32 = 0;
        for (entity, _turn, confused) in (&entities, &mut turns, &mut confusion).join() {
            confused.turns -= 1;
            if confused.turns < 1 {
                not_confused.push(entity);
            } else {
                not_my_turn.push(entity);
            }
            conf_turns = confused.turns;
        }

        for e in not_my_turn {
            gamelog
                .entries
                .push(format!("still confused {}", conf_turns).to_string());
            turns.remove(e);
        }

        for e in not_confused {
            gamelog.entries.push("confusion ends".to_string());
            confusion.remove(e);
        }
    }
}
