//#![allow(unused)]

use super::{
    camera, CombatStats, Equipped, GameLog, Hidden, HungerClock, HungerState, InBackpack, Map,
    Name, Player, Position, RexAssets, RunState, State,
    Viewshed, /*MAPHEIGHT, MAPWIDTH,*/
              /*SCREENHEIGHT, SCREENWIDTH,*/
};
use rltk::{/*console,*/ Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use std::cmp::max;

pub const LOGHEIGHT: u32 = 7;

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let (screen_width, screen_height) = ctx.get_char_size();

    //log box
    ctx.draw_box(
        0,
        screen_height - LOGHEIGHT - 2,
        screen_width - 1,
        LOGHEIGHT + 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let hunger = ecs.read_storage::<HungerClock>();

    for (_player, stats, hc) in (&players, &combat_stats, &hunger).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        let health_color = match stats.hp {
            20..=30 => rltk::GREEN,
            10..=19 => rltk::ORANGE,
            1..=9 => rltk::RED,
            _ => rltk::BLACK,
        };

        ctx.print_color(
            12,
            screen_height - LOGHEIGHT - 2,
            RGB::named(health_color),
            RGB::named(rltk::BLACK),
            &health,
        );

        match hc.state {
            HungerState::WellFed => ctx.print_color(
                screen_width - 9,
                screen_height - LOGHEIGHT - 1,
                RGB::named(rltk::BLUE),
                RGB::named(rltk::BLACK),
                "Well Fed",
            ),
            HungerState::Normal => {}
            HungerState::Hungry => ctx.print_color(
                screen_width - 9,
                screen_height - LOGHEIGHT - 1,
                RGB::named(rltk::ORANGE),
                RGB::named(rltk::BLACK),
                "Hungry",
            ),
            HungerState::Starving => ctx.print_color(
                screen_width - 9,
                screen_height - LOGHEIGHT - 1,
                RGB::named(rltk::RED),
                RGB::named(rltk::BLACK),
                "Starving",
            ),
        }

        ctx.draw_bar_horizontal(
            28,
            screen_height - LOGHEIGHT - 2,
            screen_width - 29,
            stats.hp,
            stats.max_hp,
            RGB::named(health_color),
            RGB::named(rltk::BLACK),
        );

        let log = ecs.fetch::<GameLog>();
        let mut log_y: i32 = screen_height as i32 - 2;
        for s in log.entries.iter().rev() {
            if log_y > (screen_height - LOGHEIGHT - 2) as i32 {
                ctx.print(2, log_y, s);
            }
            log_y -= 1;
        }
    }

    //mark current level
    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(
        2,
        screen_height - LOGHEIGHT - 2,
        RGB::named(rltk::AQUA),
        RGB::named(rltk::BLACK),
        &depth,
    );

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));

    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let (screen_width, _screen_height) = ctx.get_char_size();

    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;

    if mouse_map_pos.0 >= map.width
        || mouse_map_pos.1 >= map.height
        || mouse_map_pos.0 < 1
        || mouse_map_pos.1 < 1
    {
        return;
    }
    if !map.visible_tiles[map.xy_idx(mouse_map_pos.0, mouse_map_pos.1)] {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position, _hidden) in (&names, &positions, !&hidden).join() {
        if position.x == mouse_map_pos.0 && position.y == mouse_map_pos.1 {
            let pos_string = format!("{} {}", position.x, position.y);
            tooltip.push(name.name.to_string());
            tooltip.push(pos_string);
        }
    }

    if !tooltip.is_empty() {
        //get width of longest string in tooltip
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        //pad for " ->" or "<- "
        width += 3;

        if mouse_pos.0 > screen_width as i32 / 2 {
            //item to right of player, draw to left of item
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;

            ctx.draw_box(
                arrow_pos.x - width + 1,
                arrow_pos.y - 1,
                width - 2,
                tooltip.len() + 1,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
            );

            for s in tooltip.iter() {
                ctx.print_color(
                    left_x,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                    s,
                );
                let padding = width - 3 - s.len() as i32;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - 2 - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::BLACK),
                        " ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                "->".to_string(),
            );
        } else {
            //item to left of player, draw to right of item

            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            ctx.draw_box(
                arrow_pos.x + 2,
                arrow_pos.y - 1,
                width - 2,
                tooltip.len() + 1,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
            );

            for s in tooltip.iter() {
                ctx.print_color(
                    left_x + 1,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                    s,
                );
                let padding = width - 3 - s.len() as i32;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + s.len() as i32 + 3 + i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::BLACK),
                        " ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                "<-".to_string(),
            );
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

//TO_FIX: parameterize in terms of screen dimensions
pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Inventory",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _pack, name)) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .enumerate()
    {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, name.name.to_string());
        equippable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Drop Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();

    for (j, (entity, _pack, name)) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .enumerate()
    {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, name.name.to_string());
        equippable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn remove_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Remove Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _pack, name)) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .enumerate()
    {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, name.name.to_string());
        equippable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut Rltk,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(&gs.ecs, ctx);
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Select Target:",
    );

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                let screen_x = idx.x - min_x;
                let screen_y = idx.y - min_y;
                if screen_x > 1
                    && screen_x < (max_x - min_x) - 1
                    && screen_y > 1
                    && screen_y < (max_y - min_y) - 1
                {
                    ctx.set_bg(screen_x, screen_y, RGB::named(rltk::BLUE));
                    available_cells.push(idx);
                }
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;

    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_map_pos.0 && idx.y == mouse_map_pos.1 {
            valid_target = true;
        }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));

        let return_key_hit: bool = match ctx.key {
            None => false,

            Some(VirtualKeyCode::Return) => true,
            _ => false,
        };

        if ctx.left_click || return_key_hit {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}
#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    let (screen_width, screen_height) = ctx.get_char_size();

    let line1 = "Your journey has ended!";
    let line2 = "One day, we'll tell you all about how you did.";
    let line3 = "That day, sadly, is not in this chapter...";
    let line4 = "Press any key to return to the menu.";

    let text_width = max(line1.len(), max(line2.len(), max(line3.len(), line4.len())));
    let menu_width = 4 + text_width + text_width % 2;
    let menu_height = 10;
    let x_offset = (screen_width as usize - menu_width) / 2;
    let y_offset = (screen_height as usize - menu_height) / 2;

    ctx.draw_box_double(
        x_offset - 1,
        y_offset,
        menu_width,
        menu_height,
        RGB::named(rltk::WHEAT),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color_centered(
        y_offset + 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        line1,
    );
    ctx.print_color_centered(
        y_offset + 4,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        line2,
    );
    ctx.print_color_centered(
        y_offset + 5,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        line3,
    );

    ctx.print_color_centered(
        y_offset + 7,
        RGB::named(rltk::MAGENTA),
        RGB::named(rltk::BLACK),
        line4,
    );

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let (screen_width, screen_height) = ctx.get_char_size();

    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    let assets = gs.ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    let line1 = "Rust Roguelike Tutorial";
    let line2 = "by chmonster";
    let line3 = "Use Up/Down Arrows and Enter";
    let line4 = "Begin New Game";
    let line5 = "Load Game";
    let line6 = "Quit";

    let menu_height = 10;
    let line_width = max(
        line1.len(),
        max(
            line2.len(),
            max(line3.len(), max(line4.len(), max(line5.len(), line6.len()))),
        ),
    );
    let menu_width = 4 + line_width + line_width % 2;
    let x_offset = (screen_width as usize - menu_width) / 2;
    let y_offset = screen_height - menu_height - 2;

    ctx.draw_box_double(
        x_offset - 1,
        y_offset,
        menu_width,
        menu_height,
        RGB::named(rltk::WHEAT),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color_centered(
        y_offset + 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        line1,
    );
    ctx.print_color_centered(
        y_offset + 3,
        RGB::named(rltk::CYAN),
        RGB::named(rltk::BLACK),
        line2,
    );
    ctx.print_color_centered(
        y_offset + 4,
        RGB::named(rltk::GRAY),
        RGB::named(rltk::BLACK),
        line3,
    );

    let mut y = y_offset + 6;
    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(y, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), line4);
        } else {
            ctx.print_color_centered(y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), line4);
        }
        y += 1;

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::MAGENTA),
                    RGB::named(rltk::BLACK),
                    line5,
                );
            } else {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                    line5,
                );
            }
            y += 1;
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(y, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), line6);
        } else {
            ctx.print_color_centered(y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), line6);
        }

        match ctx.key {
            None => {
                return MainMenuResult::NoSelection {
                    selected: selection,
                }
            }
            Some(key) => match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    }
                }
                VirtualKeyCode::Up => {
                    let mut newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !save_exists {
                        newselection = MainMenuSelection::NewGame;
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Down => {
                    let mut newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !save_exists {
                        newselection = MainMenuSelection::Quit;
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Return => {
                    return MainMenuResult::Selected {
                        selected: selection,
                    }
                }
                _ => {
                    return MainMenuResult::NoSelection {
                        selected: selection,
                    }
                }
            },
        }
    }

    MainMenuResult::NoSelection {
        selected: MainMenuSelection::NewGame,
    }
}
