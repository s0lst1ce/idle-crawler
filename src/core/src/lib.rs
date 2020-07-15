mod buildings;
pub mod clock;
mod player;
mod pos;
pub mod resources;
pub mod response;
mod tile;
pub(crate) mod trade;
pub use self::buildings::{load_buildings, AllBuildings, Building, BuildingID, DependencyTree};
use self::clock::Clock;
pub use self::player::{Generator, Player, Username};
pub use self::pos::PosGenerator;
pub use self::resources::{load_resources, AllResources, ResourceID};
pub use self::response::{Action, Event, Exception, Response, World};
pub use self::tile::{Position, Tile};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{write, File};
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const BUILDINGS_PATH: &str = "data/buildings.json";
const RESOURCES_PATH: &str = "data/resources.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    //we need to make sure they stay ordered to enable correct behavior of PosGenerator
    pub world: HashMap<Position, Tile>,
    pub players: HashMap<Username, Player>,
    pos_gen: PosGenerator,
}

#[derive(Debug)]
pub struct Game {
    pub data: GameData,
    buildings: AllBuildings,
    resources: AllResources,
    dep_tree: DependencyTree,
    master: (Sender<Exception>, Receiver<(Username, Event)>),
}

impl Game {
    pub fn run(&mut self, ups: u8) -> Result<()> {
        let mut i = 0;
        let mut clock = Clock::new(ups);
        loop {
            i += 1;
            self.update()?;
            thread::sleep(clock.tick());
            println!("\nIteration {:?}", i);
            println!("Players {:?}", self.get_players());
        }
    }

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

    pub fn load(
        &self,
        path: PathBuf,
        master: (Sender<Exception>, Receiver<(Username, Event)>),
    ) -> Result<Game, io::Error> {
        let mut file = String::new();
        File::open(path)?.read_to_string(&mut file)?;
        let data: GameData = serde_json::from_str(&file)?;
        let (buildings, tree) = load_buildings(BUILDINGS_PATH);
        Ok(Game {
            data,
            buildings: buildings,
            dep_tree: tree,
            resources: load_resources(RESOURCES_PATH),
            master: master,
        })
    }

    pub fn new(nbr: u32, master: (Sender<Exception>, Receiver<(Username, Event)>)) -> Game {
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
            master: master,
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

    /// Gets a tile for any position
    ///
    /// Either returns a reference to an existing tile or makes a new one.
    pub fn get_tile(&mut self, pos: Position) -> Tile {
        todo!()
    }

    ///Processes player action events.
    //This method expects that the username exists. Panics otherwise.
    //The username should be provided by the binary (not the lib) and determined by the socket address.
    pub fn process(
        &mut self,
        username: &Username,
        event: Event,
    ) -> Result<Option<Event>, Exception> {
        let player = match self.data.players.get_mut(username) {
            Some(player) => player,
            None => return Err(Exception::Unregistered),
        };
        //I need to make sure the player exists. Here or elsewhere?
        match event {
            Event::Player(action) => match action {
                Action::Build {
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
                        return Err(Exception::PlaceHolder);
                    } else {
                        Ok(None)
                    }
                }

                Action::Demolish {
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
                        return Err(Exception::PlaceHolder);
                    } else {
                        Ok(None)
                    }
                }
                Action::Hire { building, amount } => {
                    if let Err(_) = player.hire(building, amount) {
                        return Err(Exception::PlaceHolder);
                    } else {
                        Ok(None)
                    }
                }
                Action::Fire { building, amount } => {
                    if let Err(_) = player.fire(building, amount) {
                        return Err(Exception::PlaceHolder);
                    } else {
                        Ok(None)
                    }
                }
                Action::Deposit { resource, amount } => {
                    if let Err(_) = player.deposit(resource, amount) {
                        Err(Exception::PlaceHolder)
                    } else {
                        Ok(None)
                    }
                }
                Action::Withdraw { resource, amount } => {
                    if let Err(_) = player.withdraw(resource, amount) {
                        return Err(Exception::PlaceHolder);
                    } else {
                        Ok(None)
                    }
                }
                Action::Trade { from, to, offer } => unimplemented!(),
            },
            Event::World(world) => match world {
                World::GetTile(pos) => {
                    if player.lands.contains(&pos) {
                        return Ok(Some(Event::World(World::Tile(self.get_tile(pos)))));
                    } else {
                        return Err(Exception::PlaceHolder);
                    }
                }
                World::Tile(tile) => todo!(), //we don't have the position anymore, what do to :thinking:
            },
        }
    }
}
