mod buildings;
mod player;
mod pos;
mod tile;
pub use self::buildings::{load_buildings, Building};
pub use self::player::Player;
pub use self::pos::PosGenerator;
pub use self::tile::{Position, Tile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{write, File};
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
struct GameData {
    //we need to make sure they stay ordered to enable correct behavior of PosGenerator
    world: HashMap<Position, Tile>,
    //HashMap<"username", Player>
    players: HashMap<String, Player>,
    pos_gen: PosGenerator,
}

#[derive(Debug)]
pub struct Game {
    data: GameData,
    buildings: HashMap<String, Building>,
}

impl Game {
    pub fn save(&self, path: PathBuf) -> Result<(), io::Error> {
        // create a save file with the current time as filename
        let now = SystemTime::now();
        write(
            path.join(
                now.duration_since(UNIX_EPOCH)
                    .expect("Time shouldn't rewind.")
                    .as_secs()
                    .to_string(),
            )
            .join(".json"),
            serde_json::to_string(&self.data)?,
        )?;
        Ok(())
    }

    pub fn load(&self, path: PathBuf) -> Result<Game, io::Error> {
        let mut file = String::new();
        File::open(path)?.read_to_string(&mut file)?;
        let data: GameData = serde_json::from_str(&file)?;
        Ok(Game {
            data,
            buildings: load_buildings(),
        })
    }

    pub fn new(nbr: u32) -> Game {
        let mut world = HashMap::new();
        let mut pos_gen = PosGenerator::new(0);
        for _ in 0..nbr {
            world.insert(pos_gen.next().unwrap(), Tile::new());
        }
        Game {
            data: GameData {
                world: world,
                players: HashMap::new(),
                pos_gen: pos_gen,
            },
            buildings: load_buildings(),
        }
    }
}
