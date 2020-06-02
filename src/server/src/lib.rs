use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
    //we need to make sure they stay ordered to enable correct behavior of PosGenerator
    world: HashMap<Position, Tile>,
    //HashMap<"username", Player>
    players: HashMap<String, Player>,
}

const VERTICE_COUPLE: [Position; 2] = [Position { x: 1, y: 0 }, Position { x: 0, y: -1 }];

#[derive(Debug)]
struct PosGenerator {
    step: u32,
    last_pos: Position,
    tiles: Vec<Position>,
    vertices: Vec<Position>,
}

impl PosGenerator {
    pub fn new(step: u32) -> PosGenerator {
        PosGenerator {
            step: step,
            tiles: Vec::new(),
            vertices: Vec::new(),
            last_pos: Position { x: 0, y: 0 },
        }
    }
    fn total_pos(&self) -> u32 {
        let nbr = 1;
        for i in 0..self.step {
            nbr += 2 * i;
        }
        nbr
    }

    fn next_vertices(&mut self) -> () {
        let mut sign: i32 = 1;
        for _ in 0..2 {
            for i in 0..2 {
                self.vertices.push(Position {
                    x: VERTICE_COUPLE[i].x * sign * self.step as i32,
                    y: VERTICE_COUPLE[i].y * sign * self.step as i32,
                });
            }
            sign *= -1;
        }
    }

    fn next_tiles(&mut self) -> () {
        if self.vertices.len() == 0 {
            self.next_vertices();
        }
        let Position { x, y } = self.vertices.remove(0);
        for i in 0..(x | y) {
            self.tiles.push(Position {
                x: (self.last_pos.x | self.last_pos.x + i),
                y: (self.last_pos.y | self.last_pos.y + i),
            })
        }
    }
}

impl Iterator for PosGenerator {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        if self.tiles.len() == 0 {
            self.next_tiles();
        }
        Some(self.tiles.remove(0))
    }
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

    pub fn load(&self, path: PathBuf) -> Result<Game, io::Error> {
        let file = File::open(path)?;
        let game: Game = serde_json::from_reader(file)?;
        Ok(game)
    }

    pub fn new(nbr: u32) -> Game {
        let world = HashMap::new();
        for _ in 0..nbr {
            tiles.push(Tile::new());
        }
    }
}

impl Tile {
    fn new() -> Tile {
        unimplemented!()
    }
}
