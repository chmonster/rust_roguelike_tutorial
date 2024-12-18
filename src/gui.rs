//#![allow(unused)]
use super::{
    camera, camera::VIEWHEIGHT, camera::VIEWWIDTH, gamelog, gamelog::LOGHEIGHT, Attribute,
    Attributes, Consumable, CursedItem, Duration, Equipped, Hidden, HungerClock, HungerState,
    InBackpack, Item, KnownSpells, MagicItem, MagicItemClass, Map, MasterDungeonMap, Name,
    ObfuscatedName, Pools, RexAssets, RunState, State, StatusEffect, Vendor, VendorMode, Viewshed,
    Weapon, SCREENHEIGHT, SCREENWIDTH,
};
use rltk::{Point, Rltk, TextBlock, VirtualKeyCode, RGB};
use specs::prelude::*;
use std::cmp::max;

pub const STATHEIGHT: u32 = 9;

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    ResumeGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

#[derive(PartialEq, Copy, Clone)]
pub enum VendorResult {
    NoResponse,
    Cancel,
    Sell,
    BuyMode,
    SellMode,
    Buy,
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

    // Overall box
    draw_hollow_box(
        ctx,
        0,
        0,
        SCREENWIDTH as i32 - 1,
        SCREENHEIGHT as i32 - 1,
        box_gray,
        black,
    );
    // Map box
    draw_hollow_box(
        ctx,
        0,
        0,
        VIEWWIDTH as i32 + 1,
        VIEWHEIGHT as i32 + 2,
        box_gray,
        black,
        //RGB::named(rltk::GREEN),
    );
    // Log box
    draw_hollow_box(
        ctx,
        0,
        VIEWHEIGHT as i32 + 2,
        SCREENWIDTH as i32 - 1,
        LOGHEIGHT as i32 + 1,
        box_gray,
        black,
        //RGB::named(rltk::BLUE),
    );
    // Stat box
    draw_hollow_box(
        ctx,
        VIEWWIDTH as i32 + 1,
        0,
        (SCREENWIDTH - VIEWWIDTH) as i32 - 2,
        STATHEIGHT as i32 + 1,
        box_gray,
        black,
        //RGB::named(rltk::YELLOW),
    );

    //box connectors
    ctx.set(0, VIEWHEIGHT + 2, box_gray, black, to_cp437('├'));
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
        VIEWHEIGHT + 2,
        box_gray,
        black,
        to_cp437('┴'),
    );
    ctx.set(
        SCREENWIDTH - 1,
        STATHEIGHT + 2,
        box_gray,
        black,
        to_cp437('┤'),
    );
    ctx.set(
        SCREENWIDTH - 1,
        VIEWHEIGHT + 2,
        box_gray,
        black,
        to_cp437('┤'),
    );

    // Draw level ID
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
    let map_label = format!("{}: {}", map.depth, map.name);
    ctx.print_color(x_pos + 1, 0, white, black, &map_label);
    std::mem::drop(map);

    // Draw stats
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();

    let hb3: i32 = player_pools.hit_points.max;
    let hb2: i32 = player_pools.hit_points.max * 2 / 3;
    let hb1: i32 = player_pools.hit_points.max / 3;
    let chp = player_pools.hit_points.current;

    let health_color = if player_pools.god_mode {
        rltk::WHITE
    } else {
        match chp {
            chp if chp < 1 => rltk::BLACK,
            chp if chp < hb1 => rltk::RED,
            chp if chp < hb2 => rltk::ORANGE,
            chp if chp < hb3 => rltk::GREEN,
            _ => rltk::GREEN,
        }
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

    // Initiative and weight and gold
    ctx.print_color(
        50,
        STATHEIGHT + 2,
        white,
        black,
        format!(
            "{:.0} lbs ({} lbs max)",
            player_pools.total_weight,
            (attr.might.base + attr.might.modifiers) * 15
        ),
    );
    ctx.print_color(
        50,
        STATHEIGHT + 3,
        white,
        black,
        format!(
            "Initiative Penalty: {:.0}",
            player_pools.total_initiative_penalty
        ),
    );
    ctx.print_color(
        50,
        STATHEIGHT + 4,
        rltk::RGB::named(rltk::GOLD),
        black,
        format!("Gold: {:.1}", player_pools.gold),
    );

    // Equipped
    let yellow = RGB::named(rltk::YELLOW);
    let mut equipment_y = STATHEIGHT + 6;
    let equipped = ecs.read_storage::<Equipped>();
    let entities = ecs.entities();
    let weapon = ecs.read_storage::<Weapon>();
    for (entity, equipped_by) in (&entities, &equipped).join() {
        if equipped_by.owner == *player_entity {
            let name = get_item_display_name(ecs, entity);
            ctx.print_color(
                VIEWWIDTH + 2,
                equipment_y,
                get_item_color(ecs, entity),
                black,
                &name,
            );
            equipment_y += 1;

            if let Some(weapon) = weapon.get(entity) {
                let mut weapon_info = match weapon.damage_bonus {
                    n if n < 0 => {
                        format!(
                            "┤ {} ({}d{}{})",
                            &name,
                            weapon.damage_n_dice,
                            weapon.damage_die_type,
                            weapon.damage_bonus
                        )
                    }
                    0 => {
                        format!(
                            "┤ {} ({}d{})",
                            &name, weapon.damage_n_dice, weapon.damage_die_type
                        )
                    }
                    _ => {
                        format!(
                            "┤ {} ({}d{}+{})",
                            &name,
                            weapon.damage_n_dice,
                            weapon.damage_die_type,
                            weapon.damage_bonus
                        )
                    }
                };

                if let Some(range) = weapon.range {
                    weapon_info += &format!(" (range: {}, F to fire, V cycles targets)", range);
                }
                weapon_info += " ├";
                ctx.print_color(3, VIEWHEIGHT + 2, yellow, black, &weapon_info);
            }
        }
    }

    // Consumables
    let mut consumable_y = equipment_y + 2;
    //let green = RGB::from_f32(0.0, 1.0, 0.0);

    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut index = 1;
    let keymarker = if cfg!(unix) { "Ctrl-" } else { "Shift-" };

    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity && index < 10 {
            ctx.print_color(
                VIEWWIDTH + 2,
                consumable_y,
                yellow,
                black,
                format!("{}{}", &keymarker, index),
            );

            ctx.print_color(
                VIEWWIDTH + keymarker.len() as u32 + 4,
                consumable_y,
                get_item_color(ecs, entity),
                black,
                get_item_display_name(ecs, entity),
            );
            consumable_y += 1;
            index += 1;
        }
    }

    // Spells
    consumable_y += 1;
    let blue = RGB::named(rltk::CYAN);
    let known_spells_storage = ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;
    for spell in known_spells.iter() {
        ctx.print_color(
            VIEWWIDTH + 2,
            consumable_y,
            blue,
            black,
            format!("{}{}", &keymarker, index),
        );
        ctx.print_color(
            VIEWWIDTH + keymarker.len() as u32 + 4,
            consumable_y,
            blue,
            black,
            format!("{} ({})", spell.display_name, spell.mana_cost),
        );
        index += 1;
        consumable_y += 1;
    }

    // Status
    let mut status_y = VIEWHEIGHT;
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();

    match hc.state {
        HungerState::WellFed => {
            ctx.print_color(
                VIEWWIDTH + 2,
                status_y,
                RGB::named(rltk::GREEN),
                RGB::named(rltk::BLACK),
                "Well Fed",
            );
            status_y -= 1;
        }
        HungerState::Normal => {}
        HungerState::Hungry => {
            ctx.print_color(
                VIEWWIDTH + 2,
                status_y,
                RGB::named(rltk::ORANGE),
                RGB::named(rltk::BLACK),
                "Hungry",
            );
            status_y -= 1;
        }
        HungerState::Starving => {
            ctx.print_color(
                VIEWWIDTH + 2,
                status_y,
                RGB::named(rltk::RED),
                RGB::named(rltk::BLACK),
                "Starving",
            );
            status_y -= 1;
        }
    }

    let statuses = ecs.read_storage::<StatusEffect>();
    let durations = ecs.read_storage::<Duration>();
    let names = ecs.read_storage::<Name>();
    for (status, duration, name) in (&statuses, &durations, &names).join() {
        if status.target == *player_entity {
            ctx.print_color(
                VIEWWIDTH + 2,
                status_y,
                RGB::named(rltk::RED),
                RGB::named(rltk::BLACK),
                format!("{} ({})", name.name, duration.turns),
            );
            status_y -= 1;
        }
    }
    if player_pools.god_mode {
        ctx.print_color(
            VIEWWIDTH + 2,
            status_y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            "God Mode",
        );
        //status_y -= 1;
    }

    // Draw the log
    let mut block = TextBlock::new(
        1,
        SCREENHEIGHT as i32 - LOGHEIGHT as i32 - 1,
        SCREENWIDTH as i32 - 2,
        LOGHEIGHT as i32,
    );
    block
        .print(&gamelog::log_display())
        .expect("log display out of space");
    block.render(&mut rltk::BACKEND_INTERNAL.lock().consoles[0].console);

    // let log = ecs.fetch::<GameLog>();
    // let mut log_y = SCREENHEIGHT as i32 - 2;
    // for s in log.entries.iter().rev() {
    //     if log_y > (SCREENHEIGHT as i32 - log_height as i32 + 1) {
    //         ctx.print(2, log_y, s);
    //     }
    //     log_y -= 1;
    // }

    // Draw mouse cursor
    //TODO : why was this removed?
    //let mouse_pos = ctx.mouse_pos();
    //ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));

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
    use rltk::Algorithm2D;

    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    //let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();
    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    //let entities = ecs.entities();

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

    if !map.in_bounds(rltk::Point::new(mouse_map_pos.0, mouse_map_pos.1)) {
        return;
    }
    let mouse_idx = map.xy_idx(mouse_map_pos.0, mouse_map_pos.1);
    if !map.visible_tiles[mouse_idx] {
        return;
    }

    let mut tip_boxes: Vec<Tooltip> = Vec::new();

    crate::spatial::for_each_tile_content(mouse_idx, |entity| {
        // for entity in map.tile_content[mouse_idx]
        //     .iter()
        //     .filter(|e| hidden.get(**e).is_none())
        // {
        //for (entity, position, _hidden) in (&entities, &positions, !&hidden).join() {
        if hidden.get(entity).is_some() {
            return;
        }

        let mut tip = Tooltip::new();
        tip.add(get_item_display_name(ecs, entity));

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

        // Status effects
        let statuses = ecs.read_storage::<StatusEffect>();
        let durations = ecs.read_storage::<Duration>();
        let names = ecs.read_storage::<Name>();
        for (status, duration, name) in (&statuses, &durations, &names).join() {
            if status.target == entity {
                tip.add(format!("{} ({})", name.name, duration.turns));
            }
        }

        #[cfg(debug_assertions)]
        {
            tip.add(format!("{}, {}", mouse_map_pos.0, mouse_map_pos.1));
        }

        tip_boxes.push(tip);
    });

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
    for (j, (entity, _pack)) in (&entities, &backpack)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .enumerate()
    {
        //index
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
        //name
        ctx.print_color(
            21,
            y,
            get_item_color(&gs.ecs, entity),
            RGB::from_f32(0.0, 0.0, 0.0),
            //name.name.to_string(),
            get_item_display_name(&gs.ecs, entity),
        );

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

    for (j, (entity, _pack)) in (&entities, &backpack)
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

        //ctx.print(21, y, name.name.to_string());
        ctx.print_color(
            21,
            y,
            get_item_color(&gs.ecs, entity),
            RGB::from_f32(0.0, 0.0, 0.0),
            get_item_display_name(&gs.ecs, entity),
        );

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
    for (j, (entity, _pack)) in (&entities, &backpack)
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

        //ctx.print(21, y, name.name.to_string());
        ctx.print_color(
            21,
            y,
            get_item_color(&gs.ecs, entity),
            RGB::from_f32(0.0, 0.0, 0.0),
            get_item_display_name(&gs.ecs, entity),
        );

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

    //TODO: vectorize and iterate the lines
    let line1 = "Your journey has ended!";
    let line2 = "One day, we'll tell you all about how you did.";
    let line3 = "That day, sadly, is not in this chapter...";

    let line4 = &format!(
        "You lived for {} turns, took {} damage and inficted {} damage.",
        crate::gamelog::get_event_count("Turn"),
        crate::gamelog::get_event_count("Damage Taken"),
        crate::gamelog::get_event_count("Damage Inflicted")
    );

    let line5 = "Press any key to return to the menu.";

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
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        line4,
    );

    ctx.print_color_centered(
        y_offset + 9,
        RGB::named(rltk::MAGENTA),
        RGB::named(rltk::BLACK),
        line5,
    );

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let (screen_width, screen_height) = ctx.get_char_size();

    let save_exists = super::saveload::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    let assets = gs.ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    let title = "Rust Roguelike Tutorial";
    let byline = "by chmonster";
    let directions = "Use Up/Down Arrows and Enter";

    let opt_resume = if gs.new_game {
        "Start New Game"
    } else {
        "Resume Game"
    };
    let opt_load = "Load Game";
    let opt_quit = "Quit";

    let menu_height = 10;
    let line_width = max(
        title.len(),
        max(
            byline.len(),
            max(
                directions.len(),
                max(opt_resume.len(), max(opt_load.len(), opt_quit.len())),
            ),
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
        title,
    );
    ctx.print_color_centered(
        y_offset + 3,
        RGB::named(rltk::CYAN),
        RGB::named(rltk::BLACK),
        byline,
    );
    ctx.print_color_centered(
        y_offset + 4,
        RGB::named(rltk::GRAY),
        RGB::named(rltk::BLACK),
        directions,
    );

    let mut y = y_offset + 6;
    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        if selection == MainMenuSelection::ResumeGame {
            ctx.print_color_centered(
                y,
                RGB::named(rltk::MAGENTA),
                RGB::named(rltk::BLACK),
                opt_resume,
            );
        } else {
            ctx.print_color_centered(
                y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                opt_resume,
            );
        }
        y += 1;

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::MAGENTA),
                    RGB::named(rltk::BLACK),
                    opt_load,
                );
            } else {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                    opt_load,
                );
            }
            y += 1;
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(
                y,
                RGB::named(rltk::MAGENTA),
                RGB::named(rltk::BLACK),
                opt_quit,
            );
        } else {
            ctx.print_color_centered(
                y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                opt_quit,
            );
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
                        MainMenuSelection::ResumeGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::ResumeGame,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !save_exists {
                        newselection = MainMenuSelection::ResumeGame;
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Down => {
                    let mut newselection;
                    match selection {
                        MainMenuSelection::ResumeGame => newselection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::ResumeGame,
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
        selected: MainMenuSelection::ResumeGame,
    }
}
#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult {
    NoResponse,
    Cancel,
    TeleportToExit,
    Heal,
    Reveal,
    GodMode,
}

pub fn show_cheat_mode(_gs: &mut State, ctx: &mut Rltk) -> CheatMenuResult {
    let count = 4;
    let mut y = 25 - (count / 2);
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
    ctx.print(21, y, "Teleport to next level");

    y += 1;
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
        rltk::to_cp437('H'),
    );
    ctx.set(
        19,
        y,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437(')'),
    );
    ctx.print(21, y, "Heal all wounds");

    y += 1;
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
        rltk::to_cp437('R'),
    );
    ctx.set(
        19,
        y,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437(')'),
    );
    ctx.print(21, y, "Reveal the map");

    y += 1;
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
        rltk::to_cp437('G'),
    );
    ctx.set(
        19,
        y,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437(')'),
    );
    ctx.print(21, y, "Toggle God Mode (No Death)");

    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::T => CheatMenuResult::TeleportToExit,
            VirtualKeyCode::H => CheatMenuResult::Heal,
            VirtualKeyCode::R => CheatMenuResult::Reveal,
            VirtualKeyCode::G => CheatMenuResult::GodMode,
            VirtualKeyCode::Escape => CheatMenuResult::Cancel,
            _ => CheatMenuResult::NoResponse,
        },
    }
}

fn vendor_sell_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    _vendor: Entity,
    _mode: VendorMode,
) -> (VendorResult, Option<Entity>, Option<String>, Option<f32>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        51,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Sell Which Item? (space to switch to buy mode)",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    //let mut j = 0;
    for (j, (entity, _pack, item)) in (&entities, &backpack, &items)
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

        //ctx.print(21, y, name.name.to_string());
        ctx.print_color(
            21,
            y,
            get_item_color(&gs.ecs, entity),
            RGB::from_f32(0.0, 0.0, 0.0),
            get_item_display_name(&gs.ecs, entity),
        );
        ctx.print(50, y, format!("{:.1} gp", item.base_value * 0.8));
        equippable.push(entity);
        y += 1;
        //j += 1;
    }

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => match key {
            VirtualKeyCode::Space => (VendorResult::BuyMode, None, None, None),
            VirtualKeyCode::Escape => (VendorResult::Cancel, None, None, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        VendorResult::Sell,
                        Some(equippable[selection as usize]),
                        None,
                        None,
                    );
                }
                (VendorResult::NoResponse, None, None, None)
            }
        },
    }
}

fn vendor_buy_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    vendor: Entity,
    _mode: VendorMode,
) -> (VendorResult, Option<Entity>, Option<String>, Option<f32>) {
    use crate::data::*;

    let vendors = gs.ecs.read_storage::<Vendor>();

    let inventory = crate::data::get_vendor_items(
        &vendors.get(vendor).unwrap().categories,
        &DATA.lock().unwrap(),
    );
    let count = inventory.len();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        51,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Buy Which Item? (space to switch to sell mode)",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    for (j, sale) in inventory.iter().enumerate() {
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

        ctx.print(21, y, &sale.0);
        ctx.print(50, y, format!("{:.1} gp", sale.1 * 1.2));
        y += 1;
    }

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => match key {
            VirtualKeyCode::Space => (VendorResult::SellMode, None, None, None),
            VirtualKeyCode::Escape => (VendorResult::Cancel, None, None, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        VendorResult::Buy,
                        None,
                        Some(inventory[selection as usize].0.clone()),
                        Some(inventory[selection as usize].1),
                    );
                }
                (VendorResult::NoResponse, None, None, None)
            }
        },
    }
}

pub fn show_vendor_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    vendor: Entity,
    mode: VendorMode,
) -> (VendorResult, Option<Entity>, Option<String>, Option<f32>) {
    match mode {
        VendorMode::Buy => vendor_buy_menu(gs, ctx, vendor, mode),
        VendorMode::Sell => vendor_sell_menu(gs, ctx, vendor, mode),
    }
}

pub fn remove_curse_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let cursed = gs.ecs.read_storage::<CursedItem>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();

    let build_cursed_iterator = || {
        (&entities, &items, &cursed)
            .join()
            .filter(|(item_entity, _item, _cursed)| {
                let mut keep = false;
                if let Some(bp) = backpack.get(*item_entity) {
                    if bp.owner == *player_entity {
                        if let Some(name) = names.get(*item_entity) {
                            if dm.identified_items.contains(&name.name) {
                                keep = true;
                            }
                        }
                    }
                }
                // It's equipped, so we know it's cursed
                if let Some(equip) = equipped.get(*item_entity) {
                    if equip.owner == *player_entity {
                        keep = true;
                    }
                }
                keep
            })
    };

    let count = build_cursed_iterator().count();

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
        "Remove Curse From Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _item, _cursed)) in build_cursed_iterator().enumerate() {
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

        ctx.print_color(
            21,
            y,
            get_item_color(&gs.ecs, entity),
            RGB::from_f32(0.0, 0.0, 0.0),
            get_item_display_name(&gs.ecs, entity),
        );
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

pub fn identify_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();
    let obfuscated = gs.ecs.read_storage::<ObfuscatedName>();

    let build_cursed_iterator = || {
        (&entities, &items).join().filter(|(item_entity, _item)| {
            let mut keep = false;
            if let Some(bp) = backpack.get(*item_entity) {
                if bp.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        if obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name)
                        {
                            keep = true;
                        }
                    }
                }
            }
            // It's equipped, so we know it's cursed
            if let Some(equip) = equipped.get(*item_entity) {
                if equip.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        if obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name)
                        {
                            keep = true;
                        }
                    }
                }
            }
            keep
        })
    };

    let count = build_cursed_iterator().count();

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
        "Identify Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _item)) in build_cursed_iterator().enumerate() {
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

        ctx.print_color(
            21,
            y,
            get_item_color(&gs.ecs, entity),
            RGB::from_f32(0.0, 0.0, 0.0),
            get_item_display_name(&gs.ecs, entity),
        );
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

pub fn get_item_color(ecs: &World, item: Entity) -> RGB {
    let dm = ecs.fetch::<crate::map::MasterDungeonMap>();

    if let Some(name) = ecs.read_storage::<Name>().get(item) {
        if ecs.read_storage::<CursedItem>().get(item).is_some()
            && dm.identified_items.contains(&name.name)
        {
            return RGB::from_f32(1.0, 0.0, 0.0);
        }
    }

    if let Some(magic) = ecs.read_storage::<MagicItem>().get(item) {
        match magic.class {
            MagicItemClass::Common => return RGB::from_f32(0.5, 1.0, 0.5),
            MagicItemClass::Rare => return RGB::from_f32(0.0, 1.0, 1.0),
            MagicItemClass::Legendary => return RGB::from_f32(0.71, 0.15, 0.93),
        }
    }
    RGB::from_f32(1.0, 1.0, 1.0)
}

pub fn get_item_display_name(ecs: &World, item: Entity) -> String {
    if let Some(name) = ecs.read_storage::<Name>().get(item) {
        if ecs.read_storage::<MagicItem>().get(item).is_some() {
            let dm = ecs.fetch::<crate::map::MasterDungeonMap>();
            if dm.identified_items.contains(&name.name) {
                if let Some(c) = ecs.read_storage::<Consumable>().get(item) {
                    if c.max_charges > 1 {
                        format!("{} ({})", name.name.clone(), c.charges).to_string()
                    } else {
                        name.name.clone()
                    }
                } else {
                    name.name.clone()
                }
            } else if let Some(obfuscated) = ecs.read_storage::<ObfuscatedName>().get(item) {
                obfuscated.name.clone()
            } else {
                "Unidentified magic item".to_string()
            }
        } else {
            name.name.clone()
        }
    } else {
        "Nameless item (bug)".to_string()
    }
}
