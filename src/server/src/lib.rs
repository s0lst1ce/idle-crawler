use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

//tile position
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Position {
    x: i32,
    y: i32,
}

pub struct Client {
    //None if the user hasn't been authentificated
    username: Option<String>,
    //the tiles for which information has to be sent
    watching: Vec<Position>,
}

impl Client {
    fn new() -> Client {
        Client {
            username: None,
            watching: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Resources {
    //HashMap<"resource_name", [u8;2]>
    slots: HashMap<String, [u8; 2]>,
    //HashMap<"resource_name", u32>
    amounts: HashMap<String, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tile {
    //WARN! this does not exactly respect the tile schema descriped in the repo --> the tuple would need to be a HashMap, however this is not very elegant in rust -> needs to be thought of again
    resources: Resources,
    //Vec<"username">
    players: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Building {
    total: u32,
    workers: [u32; 2],
    tiles: HashMap<Position, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerPeople {
    //here the definition in the repo requires a mapping (ie: HashMap) -> this makes it easier to build upon but less elegant
    //we respectively have `idle`, `total` and `maximum` -> consider making it an array instead
    idle: u32,
    total: u32,
    maximum: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    //HashMap<"building_name", Building>
    buildings: HashMap<String, Building>,
    people: PlayerPeople,
}

#[derive(Debug, Serialize, Deserialize)]
struct Game {
    world: HashMap<Position, Tile>,
    //HashMap<"username", Player>
    players: HashMap<String, Player>,
}

impl Game {
    pub fn save(&self, path: PathBuf) -> Result<(), io::Error> {
        // create a save file with the current time as filename
        let now = SystemTime::now();
        let file = File::create(
            path.join(
                now.duration_since(UNIX_EPOCH)
                    .expect("Time shouldn't rewind.")
                    .as_secs()
                    .to_string(),
            )
            .join(".json"),
        )?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }
}
