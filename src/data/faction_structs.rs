use serde::Deserialize;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Reaction {
    Ignore,
    Attack,
    Flee,
}

#[derive(Deserialize, Debug)]
pub struct FactionInfo {
    pub name: String,
    pub responses: HashMap<String, String>,
}
