use crate::resources::ResourceID;
use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
//tile position
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerResources {
    //HashMap<"resource_name", [u8;2]>
    slots: HashMap<ResourceID, [u8; 2]>,
    //HashMap<"resource_name", u32>
    amounts: HashMap<ResourceID, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    resources: PlayerResources,
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
