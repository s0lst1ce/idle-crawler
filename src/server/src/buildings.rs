use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Building {
    //resource_name, amount
    pub produced: HashMap<String, u32>,
    //resource_name, amount
    pub consumed: HashMap<String, u32>,
    pub max_workers: u32,
    //resource_name, amount
    pub requisites: HashMap<String, u32>,
}

pub type AllBuildings = HashMap<String, Building>;

pub fn load_buildings<P: AsRef<Path>>(path: P) -> AllBuildings {
    let file = std::fs::read(path).expect("couldn't read buildings.json");
    let data: AllBuildings =
        serde_json::from_slice(&file).expect("couldn't serialize buildings JSON");
    data
}
