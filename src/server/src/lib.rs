use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//tile position
#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Resources {
    slots: HashMap<String, [u8; 2]>,
    amounts: HashMap<String, u32>,
}

#[derive(Debug)]
struct Tile {
    //WARN! this does not exactly respect the tile schema descriped in the repo --> the tuple would need to be a HashMap, however this is not very elegant in rust -> needs to be thought of again
    resources: Resources,
    players: HashMap<String, u32>,
}

#[derive(Debug)]
struct Building {
    total: u32,
    workers: [u32; 2],
    tiles: HashMap<Position, u32>,
}

#[derive(Debug)]
struct PlayerPeople {
    //here the definition in the repo requires a mapping (ie: HashMap) -> this makes it easier to build upon but less elegant
    //we respectively have `idle`, `total` and `maximum` -> consider making it an array instead
    idle: u32,
    total: u32,
    maximum: u32,
}

#[derive(Debug)]
struct Player {
    buildings: HashMap<String, Building>,
    people: PlayerPeople,
}

struct Game {
    world: HashMap<Position, Tile>,
    players: Vec<Player>,
}
