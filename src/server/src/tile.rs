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
struct Resources {
    //HashMap<"resource_name", [u8;2]>
    slots: HashMap<String, [u8; 2]>,
    //HashMap<"resource_name", u32>
    amounts: HashMap<String, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    //WARN! this does not exactly respect the tile schema descriped in the repo --> the tuple would need to be a HashMap, however this is not very elegant in rust -> needs to be thought of again
    resources: Resources,
    //Vec<"username">
    players: Vec<String>,
}

impl Tile {
    pub fn new() -> Tile {
        unimplemented!()
    }
}
