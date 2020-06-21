mod buildings;
pub mod clock;
mod player;
mod pos;
mod resources;
pub mod response;
mod tile;
pub use self::buildings::{load_buildings, AllBuildings, Building, BuildingID, DependencyTree};
pub use self::player::{Generator, Player};
pub use self::pos::PosGenerator;
pub use self::tile::{Position, Tile};
use crate::resources::{load_resources, AllResources};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{write, File};
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const BUILDINGS_PATH: &str = "../../utilities/buildings.json";
const RESOURCES_PATH: &str = "../../utilities/resources.json";

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
    buildings: AllBuildings,
    resources: AllResources,
    dep_tree: DependencyTree,
    //username, Generator
}

impl Game {
    pub fn get_players(&self) -> &HashMap<String, Player> {
        &self.data.players
    }

    pub fn get_buildings(&self) -> &AllBuildings {
        &self.buildings
    }

    pub fn get_resources(&self) -> &AllResources {
        &self.resources
    }

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
        let (buildings, tree) = load_buildings(BUILDINGS_PATH);
        Ok(Game {
            data,
            buildings: buildings,
            dep_tree: tree,
            resources: load_resources(RESOURCES_PATH),
        })
    }

    pub fn new(nbr: u32) -> Game {
        let mut world = HashMap::new();
        let mut pos_gen = PosGenerator::new(0);
        for _ in 0..nbr {
            world.insert(pos_gen.next().unwrap(), Tile::new());
        }
        let (buildings, tree) = load_buildings(BUILDINGS_PATH);
        Game {
            data: GameData {
                world: world,
                players: HashMap::new(),
                pos_gen: pos_gen,
            },
            buildings: buildings,
            dep_tree: tree,
            resources: load_resources(RESOURCES_PATH),
        }
    }

    pub fn update(&mut self) -> Result<()> {
        self.generate()?;
        Ok(())
    }

    pub fn generate(&mut self) -> Result<()> {
        for (_name, player) in self.data.players.iter_mut() {
            player.generate(&self.buildings, &self.dep_tree);
        }
        Ok(())
    }

    pub fn remove_player(&mut self, player: &String) -> () {
        self.data.players.remove(player);
    }

    //this for when a new player is added to the game, not to load one from the save (see Game::load)
    pub fn add_player(&mut self, player: String) -> Result<&mut Player> {
        if self.data.players.contains_key(&player) {
            Err(anyhow!("Player {} already exists", player))
        } else {
            self.data
                .players
                .insert(player.to_string(), Player::new(&self.buildings)); //we only give the buildings for testing purposes
            Ok(self.data.players.get_mut(&player).unwrap())
        }
    }
}
