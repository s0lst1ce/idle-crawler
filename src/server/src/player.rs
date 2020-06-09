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

impl Stockpile {
    fn new() -> Stockpile {
        Stockpile {
            current: 0,
            maximum: 100,
        }
    }
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
    pub fn calc_ratios(
        &self,
        tree: &DependencyTree,
        all_buildings: &AllBuildings,
    ) -> HashMap<BuildingID, f32> {
        let mut ratios: HashMap<BuildingID, f32> = HashMap::new();
        for (resource, depends) in tree.iter() {
            let mut needed = 0.0;
            for building_id in depends.iter() {
                needed += (all_buildings
                    .get(building_id)
                    .unwrap()
                    .consumed
                    .get(resource)
                    .unwrap()
                    * self.buildings.get(building_id).unwrap().workers.0)
                    as f32
                    //we may know some buildings already can't work at full efficiency
                    * match ratios.get(building_id) {
                        Some(r) => *r,
                        None => 1.0_f32,
                    };
            }
            //the maximum efficiency possible considering available current amount of resource
            let ratio: f32 =
                1.0_f32.min((needed as u32 / self.resources.get(resource).unwrap().current) as f32);
            //we apply the new ratio or keep a lower one to make sure no building will work at a higher efficiency than it can
            for building_id in depends.iter() {
                ratios
                    .entry(*building_id)
                    .and_modify(|crt_ratio| *crt_ratio = crt_ratio.min(ratio))
                    .or_insert(ratio);
            }
        }
        ratios
    }
    //true if one of the condition for a new ratios map is needed.
    //Here all we can now is whether the player has depleted a resource and efficiency needs to drop
    //The other conditions are: resource usage that cause a stockpile drop
    //and the construction/destruction of a building.
    //These **must** be checked so that the updates are consistent.
    pub fn generate(&mut self, gen: &GenMap) -> bool {
        let mut needs_change = false;
        for (resource, amount) in gen.iter() {
            self.resources
                .entry(*resource)
                .or_insert(Stockpile::new())
                .current
                .wrapping_add(*amount as u32);
        }

        needs_change
    }
}

//ideally this GenMap would be stored in the player struct
//however it shouldn't be savec in the JSON so it has to be saved eslewhere
//unless a field can be omitted by serde that is
pub type GenMap = HashMap<ResourceID, i32>;

#[derive(Debug)]
pub struct Generator {
    map: Option<GenMap>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator { map: None }
    }
    pub fn make_gen_map(
        &mut self,
        all_buildings: &AllBuildings,
        player_buildings: &HashMap<BuildingID, OwnedBuilding>,
        ratios: &HashMap<BuildingID, f32>,
    ) -> () {
        let mut gen = GenMap::new();
        for (building, ratio) in ratios.iter() {
            //adding resources produced per tick
            for (resource, amount) in all_buildings.get(building).unwrap().produced.iter() {
                *gen.entry(*resource).or_insert(0) +=
                    ((*amount * player_buildings.get(building).unwrap().workers.0) as f32 * *ratio)
                        as i32;
            }
            //substracting resources consumed per tick
            for (resource, amount) in all_buildings.get(building).unwrap().consumed.iter() {
                *gen.entry(*resource).or_insert(0) -=
                    ((*amount * player_buildings.get(building).unwrap().workers.0) as f32 * *ratio)
                        as i32;
            }
        }
        self.map = Some(gen);
        ()
    }
}
