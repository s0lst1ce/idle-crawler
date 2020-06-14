use crate::buildings::BuildingID;
use crate::resources::ResourceID;
use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
//tile position
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slots {
    pub used: u32,
    pub total: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerResources {
    //HashMap<"building_name", [u8;2]>
    pub slots: HashMap<BuildingID, Slots>,
    //HashMap<"resource_name", u32>
    amounts: HashMap<ResourceID, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    pub resources: PlayerResources,
    //Vec<"username">
    players: Vec<String>,
}

impl Tile {
    pub fn new() -> Tile {
        //currently hardcoded for testing purposes
        let mut slots = HashMap::new();
        slots.insert("iron_mine", [2, 5]);
        Tile {
            resources: PlayerResources {
                slots: HashMap::new(),
                amounts: HashMap::new(),
            },
            players: Vec::new(),
        }
    }
}
