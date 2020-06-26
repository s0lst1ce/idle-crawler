mod buildings;
pub mod clock;
mod player;
mod pos;
mod resources;
pub mod response;
mod tile;
pub use self::buildings::{load_buildings, AllBuildings, Building, BuildingID, DependencyTree};
pub use self::player::{Generator, Player, Username};
pub use self::pos::PosGenerator;
pub use self::resources::{load_resources, AllResources, ResourceID};
pub use self::response::{Event, Exception, Response};
pub use self::tile::{Position, Tile};
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
    players: HashMap<Username, Player>,
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
    pub fn get_players(&self) -> &HashMap<Username, Player> {
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

    pub fn remove_player(&mut self, player: &Username) -> () {
        self.data.players.remove(player);
    }

    //this for when a new player is added to the game, not to load one from the save (see Game::load)
    pub fn add_player(&mut self, player: Username) -> Result<&mut Player> {
        if self.data.players.contains_key(&player) {
            Err(anyhow!("Player {} already exists", player))
        } else {
            self.data
                .players
                .insert(player.to_string(), Player::new(&self.buildings)); //we only give the buildings for testing purposes
            Ok(self.data.players.get_mut(&player).unwrap())
        }
    }

    ///Processes player action events.
    //This method expects that the username exists. Panics otherwise.
    //The username should be provided by the binary (not the lib) and determined by the socket address.
    pub fn process(&mut self, username: &Username, event: Event) -> Option<Exception> {
        let player = self.data.players.get_mut(username).unwrap();
        match event {
            Event::Build {
                pos,
                building,
                amount,
            } => {
                if let Err(_) = player.build(
                    (&pos, self.data.world.get_mut(&pos).unwrap()),
                    building,
                    self.buildings.get(&building).unwrap(),
                    amount,
                ) {
                    //WARN:: this is a placeholder, the actual exception may be different but there's currently no way of determinning it.
                    return Some(Exception::PlaceHolder);
                } else {
                    None
                }
            }

            Event::Demolish {
                pos,
                building,
                amount,
            } => {
                if let Err(_) = player.demolish(
                    (&pos, self.data.world.get_mut(&pos).unwrap()),
                    building,
                    self.buildings.get(&building).unwrap(),
                    amount,
                ) {
                    //WARN:: this is a placeholder, the actual exception may be different but there's currently no way of determinning it.
                    return Some(Exception::PlaceHolder);
                } else {
                    None
                }
            }
            Event::Hire { building, amount } => {
                if let Err(_) = player.hire(building, amount) {
                    return Some(Exception::PlaceHolder);
                } else {
                    None
                }
            }
            Event::Fire { building, amount } => {
                if let Err(_) = player.fire(building, amount) {
                    return Some(Exception::PlaceHolder);
                } else {
                    None
                }
            }
            Event::Deposit { resource, amount } => {
                if let Err(_) = player.deposit(resource, amount) {
                    Some(Exception::PlaceHolder)
                } else {
                    None
                }
            }
            Event::Withdraw { resource, amount } => {
                if let Err(_) = player.withdraw(resource, amount) {
                    return Some(Exception::PlaceHolder);
                } else {
                    None
                }
            }
            Event::Discover => unimplemented!()
        }
    }
}
