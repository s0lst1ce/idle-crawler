use crate::buildings::{AllBuildings, BuildingID, DependencyTree};
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

//ideally this GenMap would be stored in the player struct
//however it shouldn't be savec in the JSON so it has to be saved eslewhere
//unless a field can be omitted by serde that is
pub type GenMap = HashMap<ResourceID, i32>;

#[derive(Debug)]
pub struct Generator {
    map: Option<GenMap>,
    ratios: HashMap<BuildingID, f32>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            map: None,
            ratios: HashMap::new(),
        }
    }

    pub fn calc_ratios(
        tree: &DependencyTree,
        all_buildings: &AllBuildings,
        player_buildings: &HashMap<BuildingID, OwnedBuilding>,
        player_resources: &HashMap<ResourceID, Stockpile>,
    ) -> HashMap<BuildingID, f32> {
        let mut ratios: HashMap<BuildingID, f32> = HashMap::new();
        for (resource, depends) in tree.iter() {
            let mut needed: u32 = 0;
            for building_id in depends.iter() {
                needed += all_buildings
                    .get(building_id)
                    .unwrap()
                    .consumed
                    .get(resource)
                    .unwrap()
                    * player_buildings.get(building_id).unwrap().workers.0;
            }
            ratios.insert(
                *resource,
                1.0_f32.min((needed / player_resources.get(resource).unwrap().current) as f32),
            );
        }
        ratios
    }

    pub fn generate(
        tree: &DependencyTree,
        all_buildings: &AllBuildings,
        player_buildings: &HashMap<BuildingID, OwnedBuilding>,
        player_resources: &HashMap<ResourceID, Stockpile>,
    ) -> GenMap {
        unimplemented!()
    }
}
