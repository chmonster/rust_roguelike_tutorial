//#![allow(dead_code)]
//#![allow(unused_imports)]

mod item_structs;
use item_structs::*;
mod mob_structs;
use mob_structs::*;
mod prop_structs;
use prop_structs::*;
mod spawn_table_structs;
use spawn_table_structs::*;
mod loot_structs;
use loot_structs::*;
mod faction_structs;
pub use faction_structs::*;
mod spell_structs;
pub use spell_structs::*;
mod weapon_traits;
pub use weapon_traits::*;

mod data_master;
pub use data_master::*;

use crate::random_table::{MasterTable, RandomTable};

use serde::Deserialize;
use std::sync::Mutex;

lazy_static! {
    pub static ref DATA: Mutex<DataMaster> = Mutex::new(DataMaster::empty());
}

rltk::embedded_resource!(DATA_FILE, "../../data/spawns.json");

struct NewMagicItem {
    name: String,
    bonus: i32,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>,
    pub spawn_table: Vec<SpawnTableEntry>,
    pub loot_tables: Vec<LootTable>,
    pub faction_table: Vec<FactionInfo>,
    pub spells: Vec<Spell>,
    pub weapon_traits: Vec<WeaponTrait>,
}

pub fn load_data() {
    rltk::link_resource!(DATA_FILE, "../../data/spawns.json");

    // Retrieve the raw data as an array of u8 (8-bit unsigned chars)
    let raw_data = rltk::embedding::EMBED
        .lock()
        .get_resource("../../data/spawns.json".to_string())
        .unwrap();
    let raw_string =
        std::str::from_utf8(raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let decoder: Data = serde_json::from_str(raw_string).expect("Unable to parse JSON");

    DATA.lock().unwrap().load(decoder);
}
