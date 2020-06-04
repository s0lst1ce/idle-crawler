use crate::resources::ResourceID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub type BuildingID = u8;

#[derive(Debug, Deserialize, Serialize)]
pub struct Building {
    pub name: String,
    //Vector of the IDs of all buildings that the player must own to build this one
    pub prerequisites: Vec<BuildingID>,
    //resource_name, amount
    pub produced: HashMap<ResourceID, u32>,
    //resource_name, amount
    pub consumed: HashMap<ResourceID, u32>,
    pub max_workers: u32,
    //resource_name, amount
    pub construction_cost: HashMap<ResourceID, u32>,
}

pub type AllBuildings = HashMap<BuildingID, Building>;

pub fn load_buildings<P: AsRef<Path>>(path: P) -> AllBuildings {
    let file = std::fs::read(path).expect("couldn't read buildings.json");
    let data: AllBuildings =
        serde_json::from_slice(&file).expect("couldn't serialize buildings JSON");
    data
}
