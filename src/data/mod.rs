//#![allow(dead_code)]
//#![allow(unused_imports)]

mod item_structs;
use item_structs::*;
mod mob_structs;
use mob_structs::*;
mod prop_structs;
use prop_structs::*;
mod data_master;
pub use data_master::*;
use serde::Deserialize;
use std::sync::Mutex;

lazy_static! {
    pub static ref DATA: Mutex<DataMaster> = Mutex::new(DataMaster::empty());
}

rltk::embedded_resource!(DATA_FILE, "../../data/spawns.json");

#[derive(Deserialize, Debug)]
pub struct Data {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>,
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

    let decoder: Data = serde_json::from_str(raw_string).expect("Unable to parse JSON");
    rltk::console::log(format!("{:?}", decoder));
}
