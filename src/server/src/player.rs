use crate::buildings::AllBuildings;
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
    workers: [u32; 2],
    tiles: HashMap<Position, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Population {
    //here the definition in the repo requires a mapping (ie: HashMap) -> this makes it easier to build upon but less elegant
    //we respectively have `idle`, `total` and `maximum` -> consider making it an array instead
    idle: u32,
    total: u32,
    maximum: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    //HashMap<"building_name", OwnedBuilding>
    buildings: HashMap<String, OwnedBuilding>,
    people: Population,
    //HashMap<"resource_name", Stockpile>
    resources: HashMap<String, Stockpile>,
}

pub type GenMap = HashMap<String, i32>;

pub fn build_gen_map(
    player_buildings: &HashMap<String, OwnedBuilding>,
    building_list: &AllBuildings,
) -> GenMap {
    let mut gen = GenMap::new();
    for (b_name, owned_building) in player_buildings.iter() {
        //adding resources produced per tick
        for (resource, amount) in building_list.get(b_name).unwrap().produced.iter() {
            *gen.entry(resource.to_string()).or_insert(0) +=
                (*amount * owned_building.workers[0]) as i32;
        }
        //substracting resources consumed per tick
        for (resource, amount) in building_list.get(b_name).unwrap().consumed.iter() {
            *gen.entry(resource.to_string()).or_insert(0) -=
                (*amount * owned_building.workers[0]) as i32;
        }
    }
    gen
}
