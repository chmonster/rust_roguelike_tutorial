use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
    UpStairs,
}

//TOFIX: newly opened doors are not walkable unless the player moves elsewhere first
pub fn tile_walkable(tt: TileType) -> bool {
    matches!(
        tt,
        TileType::Floor
            | TileType::DownStairs
            | TileType::UpStairs
            | TileType::Road
            | TileType::Grass
            | TileType::ShallowWater
            | TileType::WoodFloor
            | TileType::Bridge
            | TileType::Gravel
    )
}

pub fn tile_opaque(tt: TileType) -> bool {
    matches!(tt, TileType::Wall)
}

pub fn tile_cost(tt: TileType) -> f32 {
    match tt {
        TileType::Road => 0.8,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.2,
        _ => 1.0,
    }
}
