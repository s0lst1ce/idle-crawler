use crate::buildings::{AllBuildings, BuildingID};
use crate::resources::ResourceID;
use crate::tile::Position;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Stockpile {
    current: u32,
    maximum: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnedBuilding {
    total: u32,
    workers: (u32, u32),
    tiles: HashMap<Position, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Population {
    //here the definition in the repo requires a mapping (ie: HashMap) -> this makes it easier to build upon but less elegant
    //we respectively have `idle`, `total` and `maximum` -> consider making it an array instead
    idle: u32,
    total: u32,
    maximum: u32,
}

impl Population {
    fn new() -> Population {
        Population {
            idle: 5,
            total: 5,
            maximum: 10,
        }
    }
}

//ideally this GenMap would be stored in the player struct
//however it shouldn't be savec in the JSON so it has to be saved eslewhere
//unless a field can be omitted by serde that is
pub type GenMap = HashMap<ResourceID, i32>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    //HashMap<"building_name", OwnedBuilding>
    buildings: HashMap<BuildingID, OwnedBuilding>,
    people: Population,
    //HashMap<"resource_name", Stockpile>
    resources: HashMap<ResourceID, Stockpile>,
}

impl Player {
    pub fn new() -> Player {
        Player {
            buildings: HashMap::new(),
            people: Population::new(),
            resources: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Generator {
    map: Option<GenMap>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator { map: None }
    }

    pub fn build_gen_map(
        &mut self,
        player_buildings: &HashMap<BuildingID, OwnedBuilding>,
        building_list: &AllBuildings,
    ) -> () {
        let mut gen = GenMap::new();
        for (b_name, owned_building) in player_buildings.iter() {
            //adding resources produced per tick
            for (resource, amount) in building_list.get(b_name).unwrap().produced.iter() {
                *gen.entry(*resource).or_insert(0) += (*amount * owned_building.workers.0) as i32;
            }
            //substracting resources consumed per tick
            for (resource, amount) in building_list.get(b_name).unwrap().consumed.iter() {
                *gen.entry(*resource).or_insert(0) -= (*amount * owned_building.workers.0) as i32;
            }
        }
        self.map = Some(gen);
        ()
    }

    pub fn generate(&mut self, player: &mut Player) -> () {
        unimplemented!()
    }
}
