#![allow(unused_imports)]

use super::{map::*, spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};

mod simple_map;
use simple_map::SimpleMapBuilder;
mod rubble_map;
use rubble_map::RubbleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
use bsp_interior::BspInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod random_walk;
use random_walk::DrunkardsWalkBuilder;
mod maze;
use maze::MazeBuilder;
mod dla;
use dla::DLABuilder;
mod voronoi;
use voronoi::VoronoiCellBuilder;
mod waveform_collapse;
use waveform_collapse::WaveformCollapseBuilder;
mod prefab_builder;
use prefab_builder::{PrefabBuilder, PrefabMode};
mod common;
use common::*;
use specs::prelude::*;
mod room_based_spawner;
use room_based_spawner::RoomBasedSpawner;
mod room_based_stairs;
use room_based_stairs::RoomBasedStairs;
mod room_based_starting_position;
use room_based_starting_position::RoomBasedStartingPosition;
mod cull_unreachable;
use cull_unreachable::CullUnreachable;
mod voronoi_spawning;
use voronoi_spawning::VoronoiSpawning;
mod area_starting_point;
use area_starting_point::{AreaStartingPosition, XStart, YStart};
mod area_ending_point;
use area_ending_point::{AreaEndingPosition, XEnd, YEnd};
mod distant_exit;
use distant_exit::DistantExit;
mod room_exploder;
use room_exploder::RoomExploder;
mod round_corners;
use round_corners::RoomCornerRounder;
mod room_corridors_dogleg;
use room_corridors_dogleg::DoglegCorridors;
mod room_corridors_bsp;
use room_corridors_bsp::BspCorridors;
mod room_sorter;
use room_sorter::{RoomSort, RoomSorter};
mod room_draw;
use room_draw::RoomDrawer;
mod room_corridors_nearest;
use room_corridors_nearest::NearestCorridors;
mod room_corridors_lines;
use room_corridors_lines::StraightLineCorridors;
mod room_corridor_spawner;
use room_corridor_spawner::CorridorSpawner;
mod door_placement;
use door_placement::DoorPlacement;
mod town;
use town::town_builder;
mod forest;
use forest::forest_builder;
mod limestone_cavern;
use limestone_cavern::{
    limestone_cavern_builder, limestone_deep_cavern_builder, limestone_transition_builder,
};
mod dwarf_fort;
use dwarf_fort::*;

//marked for special builder restrictions
//must match positions in build_roll block
const RUBBLE_ID: i32 = 3;
const BSP_INTERIOR_ID: i32 = 2;

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub width: i32,
    pub height: i32,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new<S: ToString>(new_depth: i32, width: i32, height: i32, name: S) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth, width, height, name),
                starting_position: None,
                rooms: None,
                corridors: None,
                history: Vec::new(),
                width,
                height,
            },
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder."),
        };
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World, map_depth: i32) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1), map_depth);
        }
    }
}

pub fn level_builder(
    new_depth: i32,
    rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    rltk::console::log(format!("Depth: {}", new_depth));
    match new_depth {
        1 => town_builder(new_depth, rng, width, height),
        2 => forest_builder(new_depth, rng, width, height),
        3 => limestone_cavern_builder(new_depth, rng, width, height),
        4 => limestone_deep_cavern_builder(new_depth, rng, width, height),
        5 => limestone_transition_builder(new_depth, rng, width, height),
        6 => dwarf_fort_builder(new_depth, rng, width, height),
        _ => random_builder(new_depth, rng, width, height),
    }
}

fn random_start_position(rng: &mut rltk::RandomNumberGenerator) -> (XStart, YStart) {
    let xroll = rng.roll_dice(1, RUBBLE_ID);
    let x = match xroll {
        1 => XStart::Left,
        2 => XStart::Center,
        _ => XStart::Right,
    };

    let yroll = rng.roll_dice(1, RUBBLE_ID);
    let y = match yroll {
        1 => YStart::Bottom,
        2 => YStart::Center,
        _ => YStart::Top,
    };

    (x, y)
}

fn random_room_builder(rng: &mut rltk::RandomNumberGenerator, builder: &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 4);
    //let build_roll = RUBBLE_ID;
    //let build_roll = 4;
    match build_roll {
        1 => builder.start_with(BspDungeonBuilder::new()),
        BSP_INTERIOR_ID => builder.start_with(BspInteriorBuilder::new()),
        RUBBLE_ID => builder.start_with(RubbleMapBuilder::new()),
        _ => builder.start_with(SimpleMapBuilder::new()),
    }
    if build_roll != RUBBLE_ID
    //rubble is one big room and already populated
    {
        builder.with(RoomDrawer::new());
    }

    // BSP Interior still makes holes in the walls insterad of corridors; Rubble has only one room
    if build_roll != BSP_INTERIOR_ID && build_roll != RUBBLE_ID {
        // Sort by one of the 5 available algorithms
        let sort_roll = rng.roll_dice(1, 5);
        match sort_roll {
            1 => builder.with(RoomSorter::new(RoomSort::Leftmost)),
            2 => builder.with(RoomSorter::new(RoomSort::Rightmost)),
            3 => builder.with(RoomSorter::new(RoomSort::Topmost)),
            4 => builder.with(RoomSorter::new(RoomSort::Bottommost)),
            _ => builder.with(RoomSorter::new(RoomSort::Central)),
        }

        let corridor_roll = rng.roll_dice(1, 4);
        //let corridor_roll = 3;
        match corridor_roll {
            1 => builder.with(DoglegCorridors::new()),
            2 => builder.with(NearestCorridors::new()),
            3 => builder.with(StraightLineCorridors::new()),
            _ => builder.with(BspCorridors::new()),
        }

        let cspawn_roll = rng.roll_dice(1, 2);
        if cspawn_roll == 1 {
            builder.with(CorridorSpawner::new());
        }

        let modifier_roll = rng.roll_dice(1, 6);
        match modifier_roll {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            _ => {}
        }
    }

    let start_roll = rng.roll_dice(1, 2);
    match start_roll {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPosition::new(start_x, start_y));
        }
    }

    if build_roll == RUBBLE_ID {
        //single big room
        for _i in 1..rng.range(5, 8) {
            builder.with(RoomBasedStairs::new());
        }
    } else {
        let exit_roll = rng.roll_dice(1, 2);
        match exit_roll {
            1 => builder.with(RoomBasedStairs::new()),
            _ => builder.with(DistantExit::new()),
        }
    }

    let spawn_roll = rng.roll_dice(1, 2);
    match spawn_roll {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new()),
    }
}

fn random_shape_builder(rng: &mut rltk::RandomNumberGenerator, builder: &mut BuilderChain) {
    let builder_roll = rng.roll_dice(1, 14);
    match builder_roll {
        1 => builder.start_with(CellularAutomataBuilder::new()),
        2 => builder.start_with(DrunkardsWalkBuilder::open_area()),
        3 => builder.start_with(DrunkardsWalkBuilder::open_halls()),
        4 => builder.start_with(DrunkardsWalkBuilder::winding_passages()),
        5 => builder.start_with(DrunkardsWalkBuilder::fat_passages()),
        6 => builder.start_with(DrunkardsWalkBuilder::fearful_symmetry()),
        7 => builder.start_with(MazeBuilder::new()),
        8 => builder.start_with(DLABuilder::walk_inwards()),
        9 => builder.start_with(DLABuilder::walk_outwards()),
        10 => builder.start_with(DLABuilder::central_attractor()),
        11 => builder.start_with(DLABuilder::insectoid()),
        12 => builder.start_with(VoronoiCellBuilder::pythagoras()),
        13 => builder.start_with(VoronoiCellBuilder::manhattan()),
        _ => builder.start_with(PrefabBuilder::constant(
            prefab_builder::prefab_levels::WFC_POPULATED,
        )),
    }

    // Set the start to the center and cull
    builder.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    builder.with(CullUnreachable::new());

    // Now set the start to a random starting area
    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));

    // Setup an exit and spawn mobs
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
}

pub fn random_builder(
    new_depth: i32,
    rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height, "New Map");
    let type_roll = rng.roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder),
    }

    if rng.roll_dice(1, 4) == 1 {
        builder.with(WaveformCollapseBuilder::new());
        //keeps loot, player and exit positions as is.  needs repeat generation

        // Now set the start to a random starting area
        let (start_x, start_y) = random_start_position(rng);
        builder.with(AreaStartingPosition::new(start_x, start_y));

        // Setup an exit and spawn mobs
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    if rng.roll_dice(1, 7) == 1 {
        builder.with(PrefabBuilder::sectional(
            prefab_builder::prefab_sections::UNDERGROUND_FORT,
        ));
    }

    if builder.build_data.map.name != "MazeBuilder" {
        builder.with(DoorPlacement::new());
    }

    builder.with(PrefabBuilder::vaults());

    builder
}
