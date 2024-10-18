//#![allow(unused)]

use super::{
    camera, camera::VIEWHEIGHT, camera::VIEWWIDTH, Attribute, Attributes, Consumable, Equipped,
    GameLog, Hidden, HungerClock, HungerState, InBackpack, Map, Name, /*Player,*/ Pools,
    Position, RexAssets, RunState, State, Viewshed, SCREENHEIGHT, SCREENWIDTH,
};
use rltk::{/*console,*/ Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use std::cmp::max;

pub const STATHEIGHT: u32 = 9;

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

pub fn draw_hollow_box(
    console: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    use rltk::to_cp437;

    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}
fn draw_attribute(name: &str, attribute: &Attribute, y: i32, ctx: &mut Rltk) {
    let black = RGB::named(rltk::BLACK);
    let attr_gray: RGB = RGB::from_hex("#CCCCCC").expect("Oops");
    ctx.print_color(50, y, attr_gray, black, name);
    #[allow(clippy::comparison_chain)]
    let color: RGB = if attribute.modifiers < 0 {
        RGB::from_f32(1.0, 0.0, 0.0)
    } else if attribute.modifiers == 0 {
        RGB::named(rltk::WHITE)
    } else {
        RGB::from_f32(0.0, 1.0, 0.0)
    };
    ctx.print_color(
        67,
        y,
        color,
        black,
        format!("{}", attribute.base + attribute.modifiers),
    );
    ctx.print_color(73, y, color, black, format!("{}", attribute.bonus));
    if attribute.bonus > 0 {
        ctx.set(72, y, color, black, rltk::to_cp437('+'));
    }
}

#[allow(unused_imports)]
pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    use rltk::to_cp437;

    let box_gray: RGB = RGB::from_hex("#999999").expect("Oops");
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);

    let log_height = SCREENHEIGHT - VIEWHEIGHT;

    draw_hollow_box(
        ctx,
        0,
        0,
        SCREENWIDTH as i32 - 1,
        SCREENHEIGHT as i32 - 1,
        box_gray,
        black,
    ); // Overall box
    draw_hollow_box(
        ctx,
        0,
        0,
        VIEWWIDTH as i32 + 1,
        VIEWHEIGHT as i32 + 1,
        box_gray,
        black,
    ); // Map box
    draw_hollow_box(
        ctx,
        0,
        VIEWHEIGHT as i32 + 1,
        SCREENWIDTH as i32 - 1,
        (log_height - 2) as i32,
        box_gray,
        black,
    ); // Log box
    draw_hollow_box(
        ctx,
        VIEWWIDTH as i32 + 1,
        0,
        (SCREENWIDTH - VIEWWIDTH) as i32 - 2,
        STATHEIGHT as i32 + 1,
        box_gray,
        black,
    ); // Top-right panel

    //box connectors
    ctx.set(0, VIEWHEIGHT + 1, box_gray, black, to_cp437('├'));
    ctx.set(
        VIEWWIDTH + 1,
        STATHEIGHT + 1,
        box_gray,
        black,
        to_cp437('├'),
    );
    ctx.set(VIEWWIDTH + 1, 0, box_gray, black, to_cp437('┬'));
    ctx.set(
        VIEWWIDTH + 1,
        VIEWHEIGHT + 1,
        box_gray,
        black,
        to_cp437('┴'),
    );
    ctx.set(
        SCREENWIDTH - 1,
        STATHEIGHT + 1,
        box_gray,
        black,
        to_cp437('┤'),
    );
    ctx.set(
        SCREENWIDTH - 1,
        VIEWHEIGHT + 1,
        box_gray,
        black,
        to_cp437('┤'),
    );

    // Draw level name
    let map = ecs.fetch::<Map>();
    let name_length = map.name.len() + 2;
    let x_pos = (VIEWHEIGHT as i32 - 4 - name_length as i32) / 2;
    ctx.set(x_pos, 0, box_gray, black, to_cp437('┤'));
    ctx.set(
        x_pos + name_length as i32,
        0,
        box_gray,
        black,
        to_cp437('├'),
    );
    ctx.print_color(x_pos + 1, 0, white, black, &map.name);
    std::mem::drop(map);

    // Draw stats
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();

    let hb3: i32 = player_pools.hit_points.max;
    let hb2: i32 = player_pools.hit_points.max * 2 / 3;
    let hb1: i32 = player_pools.hit_points.max / 3;
    let chp = player_pools.hit_points.current;

    let health_color = match chp {
        chp if chp < 1 => rltk::BLACK,
        chp if chp < hb1 => rltk::RED,
        chp if chp < hb2 => rltk::ORANGE,
        chp if chp < hb3 => rltk::GREEN,
        _ => rltk::GREEN,
    };

    let health = format!(
        "Health: {}/{}",
        player_pools.hit_points.current, player_pools.hit_points.max
    );
    let mana = format!(
        "Mana:   {}/{}",
        player_pools.mana.current, player_pools.mana.max
    );
    let xp = format!("Level:  {}", player_pools.level);

    ctx.print_color(50, 1, white, black, &health);
    ctx.print_color(50, 2, white, black, &mana);
    ctx.print_color(50, 3, white, black, &xp);

    ctx.draw_bar_horizontal(
        64,
        1,
        14,
        player_pools.hit_points.current,
        player_pools.hit_points.max,
        RGB::named(health_color),
        RGB::named(rltk::BLACK),
    );
    ctx.draw_bar_horizontal(
        64,
        2,
        14,
        player_pools.mana.current,
        player_pools.mana.max,
        RGB::named(rltk::BLUE),
        RGB::named(rltk::BLACK),
    );

    //format!("Level:  {}", player_pools.level);
    //ctx.print_color(50, 3, white, black, &xp);
    let xp_level_start = (player_pools.level - 1) * 1000;
    ctx.draw_bar_horizontal(
        64,
        3,
        14,
        player_pools.xp - xp_level_start,
        1000,
        RGB::named(rltk::GOLD),
        RGB::named(rltk::BLACK),
    );

    // Attributes
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &attr.might, 4, ctx);
    draw_attribute("Quickness:", &attr.quickness, 5, ctx);
    draw_attribute("Fitness:", &attr.fitness, 6, ctx);
    draw_attribute("Intelligence:", &attr.intelligence, 7, ctx);
    //draw_attribute("Armor Class:", &attr.fitness, 7, ctx);

    // Equipped
    let mut equipment_y = STATHEIGHT + 2;
    let equipped = ecs.read_storage::<Equipped>();
    let name = ecs.read_storage::<Name>();
    for (equipped_by, item_name) in (&equipped, &name).join() {
        if equipped_by.owner == *player_entity {
            ctx.print_color(VIEWWIDTH + 2, equipment_y, white, black, &item_name.name);
            equipment_y += 1;
        }
    }

    // Consumables
    let mut consumable_y = equipment_y + 2;
    let green = RGB::from_f32(0.0, 1.0, 0.0);
    let yellow = RGB::named(rltk::YELLOW);
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut index = 1;
    for (carried_by, _consumable, item_name) in (&backpack, &consumables, &name).join() {
        if carried_by.owner == *player_entity && index < 10 {
            ctx.print_color(
                VIEWWIDTH + 2,
                consumable_y,
                yellow,
                black,
                format!("↑{}", index),
            );
            ctx.print_color(VIEWWIDTH + 5, consumable_y, green, black, &item_name.name);
            consumable_y += 1;
            index += 1;
        }
    }
    // Status
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state {
        HungerState::WellFed => ctx.print_color(
            VIEWWIDTH + 2,
            VIEWHEIGHT,
            RGB::named(rltk::GREEN),
            RGB::named(rltk::BLACK),
            "Well Fed",
        ),
        HungerState::Normal => {}
        HungerState::Hungry => ctx.print_color(
            VIEWWIDTH + 2,
            VIEWHEIGHT,
            RGB::named(rltk::ORANGE),
            RGB::named(rltk::BLACK),
            "Hungry",
        ),
        HungerState::Starving => ctx.print_color(
            VIEWWIDTH + 2,
            VIEWHEIGHT,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
            "Starving",
        ),
    }

    // Draw the log
    let log = ecs.fetch::<GameLog>();
    let mut log_y = SCREENHEIGHT as i32 - 2;
    for s in log.entries.iter().rev() {
        if log_y > (SCREENHEIGHT as i32 - log_height as i32 + 1) {
            ctx.print(2, log_y, s);
        }
        log_y -= 1;
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));

    draw_tooltips(ecs, ctx);
}

struct Tooltip {
    lines: Vec<String>,
}

impl Tooltip {
    fn new() -> Tooltip {
        Tooltip { lines: Vec::new() }
    }

    fn add<S: ToString>(&mut self, line: S) {
        self.lines.push(line.to_string());
    }

    fn width(&self) -> i32 {
        let mut max = 0;
        for s in self.lines.iter() {
            if s.len() > max {
                max = s.len();
            }
        }
        max as i32 + 2i32
    }

    fn height(&self) -> i32 {
        self.lines.len() as i32 + 2i32
    }

    fn render(&self, ctx: &mut Rltk, x: i32, y: i32) {
        //let box_gray: RGB = RGB::from_hex("#999999").expect("Oops");
        let light_gray: RGB = RGB::from_hex("#DDDDDD").expect("Oops");
        let white = RGB::named(rltk::WHITE);
        let black = RGB::named(rltk::BLACK);
        ctx.draw_box(x, y, self.width() - 1, self.height() - 1, white, black);
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_gray };
            ctx.print_color(x + 1, y + i as i32 + 1, col, black, s);
        }
    }
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    use rltk::to_cp437;

    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    //let (screen_width, _screen_height) = ctx.get_char_size();

    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();
    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let entities = ecs.entities();

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

    let mut tip_boxes: Vec<Tooltip> = Vec::new();

    for (entity, name, position, _hidden) in (&entities, &names, &positions, !&hidden).join() {
        if position.x == mouse_map_pos.0 && position.y == mouse_map_pos.1 {
            let mut tip = Tooltip::new();
            tip.add(name.name.to_string());

            // Comment on attributes
            let attr = attributes.get(entity);
            if let Some(attr) = attr {
                let mut s = "".to_string();
                if attr.might.bonus < 0 {
                    s += "Weak. "
                };
                if attr.might.bonus > 0 {
                    s += "Strong. "
                };
                if attr.quickness.bonus < 0 {
                    s += "Clumsy. "
                };
                if attr.quickness.bonus > 0 {
                    s += "Agile. "
                };
                if attr.fitness.bonus < 0 {
                    s += "Unheathy. "
                };
                if attr.fitness.bonus > 0 {
                    s += "Healthy."
                };
                if attr.intelligence.bonus < 0 {
                    s += "Unintelligent. "
                };
                if attr.intelligence.bonus > 0 {
                    s += "Smart. "
                };
                if s.is_empty() {
                    s = "Quite Average".to_string();
                }
                tip.add(s);
            }

            // Comment on pools
            let stat = pools.get(entity);
            if let Some(stat) = stat {
                tip.add(format!("Level: {}", stat.level));
            }

            #[cfg(debug_assertions)]
            {
                tip.add(format!("{}, {}", position.x, position.y));
            }

            tip_boxes.push(tip);
        }
    }
    if tip_boxes.is_empty() {
        return;
    }

    let box_gray: RGB = RGB::from_hex("#999999").expect("Oops");
    let white = RGB::named(rltk::WHITE);

    let arrow;
    let arrow_x;
    let arrow_y = mouse_pos.1;
    if mouse_pos.0 < VIEWWIDTH as i32 / 2 {
        // Render to the left
        arrow = to_cp437('→');
        arrow_x = mouse_pos.0 - 1;
    } else {
        // Render to the right
        arrow = to_cp437('←');
        arrow_x = mouse_pos.0 + 1;
    }
    ctx.set(arrow_x, arrow_y, white, box_gray, arrow);

    let mut total_height = 0;
    for tt in tip_boxes.iter() {
        total_height += tt.height();
    }

    let mut y = mouse_pos.1 - (total_height / 2);
    //TODO: why 50?
    while y + (total_height / 2) > 50 {
        y -= 1;
    }

    for tt in tip_boxes.iter() {
        let x = if mouse_pos.0 < VIEWWIDTH as i32 / 2 {
            mouse_pos.0 - (1 + tt.width())
        } else {
            //mouse_pos.0 + (1 + tt.width())
            mouse_pos.0 + 2
        };
        tt.render(ctx, x, y);
        y += tt.height();
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

#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult {
    NoResponse,
    Cancel,
    TeleportToExit,
}

pub fn show_cheat_mode(_gs: &mut State, ctx: &mut Rltk) -> CheatMenuResult {
    let count = 2;
    let y = 25 - (count / 2);
    ctx.draw_box(
        15,
        y - 2,
        31,
        count + 3,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Cheating!",
    );
    ctx.print_color(
        18,
        y + count + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

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
        rltk::to_cp437('T'),
    );
    ctx.set(
        19,
        y,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437(')'),
    );

    ctx.print(21, y, "Teleport to exit");

    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::T => CheatMenuResult::TeleportToExit,
            VirtualKeyCode::Escape => CheatMenuResult::Cancel,
            _ => CheatMenuResult::NoResponse,
        },
    }
}
