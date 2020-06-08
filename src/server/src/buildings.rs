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
//vector of building that consume the resource used as key
pub type DependencyTree = HashMap<ResourceID, Vec<BuildingID>>;

pub fn load_buildings<P: AsRef<Path>>(path: P) -> (AllBuildings, DependencyTree) {
    let file = std::fs::read(path).expect("couldn't read buildings.json");
    let data: AllBuildings =
        serde_json::from_slice(&file).expect("couldn't serialize buildings JSON");

    let tree = get_tree(&data);
    (data, tree)
}

fn get_tree(buildings: &AllBuildings) -> DependencyTree {
    let mut tree: DependencyTree = HashMap::new();
    for (name, building) in buildings.iter() {
        for (resource, _) in building.consumed.iter() {
            tree.entry(*resource).or_insert(Vec::new()).push(*name);
        }
    }
    tree
}
