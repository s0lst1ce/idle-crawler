use crate::buildings::{AllBuildings, Building, BuildingID, DependencyTree};
use crate::resources::ResourceID;
use crate::tile::Position;
use crate::tile::Tile;
use anyhow::{anyhow, Result};
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

impl Default for Stockpile {
    fn default() -> Self {
        Stockpile::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnedBuilding {
    total: u32,
    workers: (u32, u32),
    tiles: HashMap<Position, u32>,
}

impl OwnedBuilding {
    fn new() -> OwnedBuilding {
        OwnedBuilding {
            total: 0,
            workers: (0, 0),
            tiles: HashMap::new(),
        }
    }
}

impl Default for OwnedBuilding {
    fn default() -> Self {
        OwnedBuilding::new()
    }
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
    #[serde(skip)]
    gen: Generator,
}

impl Player {
    pub fn new() -> Player {
        let mut buildings = HashMap::new();
        buildings.insert(0, OwnedBuilding::new());

        Player {
            buildings: HashMap::new(),
            people: Population::new(),
            resources: HashMap::new(),
            gen: Generator::new(),
        }
    }

    pub fn max_buildable(
        &self,
        tiles: Vec<&Tile>,
        id: BuildingID,
        building: &Building,
        amount: u32,
    ) -> u32 {
        let mut max = u32::MAX;

        //if the building uses natural resources there must enough free slots
        //otherwise the player can't place the building
        if building.extractor {
            max = 0;
            for tile in tiles {
                if let Some(res_type) = tile.resources.slots.get(&id) {
                    max += res_type.total - res_type.used
                }
            }
        }
        for (resource, qt) in building.construction_cost.iter() {
            max = max.min(self.resources.get(resource).unwrap().current / (qt * amount));
        }
        max
    }

    //Attempts to build `amount` `building` in `tile`.
    //Fails if `amount` is too big (see max_buildable)
    //It may be useful to allow tiles to be &HashMap<Position, Tile>.Position
    //This way we could let the function choose the distribution of buildings over
    //the tiles when it doesn't matter
    pub fn build(
        &mut self,
        tiles: (&Position, &mut Tile),
        id: BuildingID,
        building: &Building,
        amount: u32,
    ) -> Result<()> {
        if amount > self.max_buildable(vec![tiles.1], id, building, amount) {
            return Err(anyhow!(format!(
                "Can't build {:?} buildings of type ID{:?}, maximum is {:?}",
                amount,
                id,
                self.max_buildable(vec![tiles.1], id, building, amount)
            )));
        }
        self.gen.needs_update = true;
        self.add_building(*tiles.0, id, building, amount);
        tiles.1.resources.slots.get_mut(&id).unwrap().used += amount;
        Ok(())
    }

    //Adds `amount` building of type `buildings` to the `pos`. Expects enough slots to be free on the `pos`.
    fn add_building(
        &mut self,
        pos: Position,
        id: BuildingID,
        building: &Building,
        amount: u32,
    ) -> () {
        let ob = self.buildings.entry(id).or_default();
        ob.total += amount;
        ob.workers.1 += amount * building.max_workers;
        *ob.tiles.entry(pos).or_default() += amount;
    }

    //Attempts to tear down `amount` `building` in `tile`.
    //Fails if `amount` is greater than the number of buildings owned by the player
    //It may be useful to allow tiles to be &HashMap<Position, Tile>.Position
    //This way we could let the function choose the distribution of buildings over
    //the tiles when it doesn't matter
    pub fn demolish(
        &mut self,
        tiles: (&Position, &mut Tile),
        id: BuildingID,
        building: &Building,
        amount: u32,
    ) -> Result<()> {
        self.gen.needs_update = true;
        self.rm_building(tiles.0, id, building, amount);
        tiles.1.resources.slots.get_mut(&id).unwrap().used -= amount;
        let mut workers = self.buildings.get_mut(&id).unwrap().workers;

        //Adjusting workers count. Workers may need to be fired.
        workers.0 -= (amount * building.max_workers) - (workers.1 - workers.0);
        workers.1 -= amount * building.max_workers;
        Ok(())
    }

    //Removes `amount` building of type `buildings` to the `pos`. Expects the player to own at least `amount` of `building`.
    fn rm_building(
        &mut self,
        pos: &Position,
        id: BuildingID,
        building: &Building,
        amount: u32,
    ) -> () {
        let ob = self.buildings.get_mut(&id).unwrap();
        ob.total -= amount;
        ob.workers.1 -= amount * building.max_workers;
        *ob.tiles.get_mut(pos).unwrap() -= amount;
    }

    //Adds `amount` of `id` resource to the player if enough place is available.
    pub fn deposit(&mut self, id: ResourceID, amount: u32) -> Result<()> {
        let mut stock = self.resources.get_mut(&id).unwrap();
        if stock.maximum - stock.current < amount {
            Err(
                anyhow! {format!("Can't add more resources than available space in {:?} stockpile!", id)},
            )
        } else {
            stock.current += amount;
            self.gen.needs_update = true;
            Ok(())
        }
    }

    //Removes `amount` of `id` resource to the player if enough is owned.
    pub fn withdraw(&mut self, id: ResourceID, amount: u32) -> Result<()> {
        let mut stock = self.resources.get_mut(&id).unwrap();
        if stock.current < amount {
            Err(anyhow! {format!("Can't remove more resource ID{:?} than the player owns!", id)})
        } else {
            stock.current -= amount;
            self.gen.needs_update = true;
            Ok(())
        }
    }

    pub fn hire(&mut self, id: BuildingID, amount: u32) -> Result<()> {
        match self.buildings.get_mut(&id) {
            Some(ob) => {
                if ob.workers.1 - ob.workers.0 < amount {
                    Err(anyhow!(format!(
                        "Tried to hire more workers than available jobs to building ID{:?}!",
                        id
                    )))
                } else {
                    ob.workers.0 += amount;
                    self.gen.needs_update = true;
                    Ok(())
                }
            }
            None => Err(anyhow!(format!(
                "Tried to add workers to building ID{:?} which the player does not own!",
                id
            ))),
        }
    }

    pub fn fire(&mut self, id: BuildingID, amount: u32) -> Result<()> {
        match self.buildings.get_mut(&id) {
            Some(ob) => {
                if ob.workers.0 < amount {
                    Err(anyhow!(format!(
                        "Tried to fire more workers than owned employees of building ID{:?}!",
                        id
                    )))
                } else {
                    ob.workers.0 -= amount;
                    self.gen.needs_update = true;
                    Ok(())
                }
            }
            None => Err(anyhow!(format!(
                "Tried to remove workers from building ID{:?} which the player does not own!",
                id
            ))),
        }
    }

    //Creates a map witht the lowest factor at which the building can work.
    //The key is the building and the value its current efficiency.
    //The later is calculated from available resources
    //and takes the production chain into account
    //It needs to be ran everytime a resource's stockpile would reach 0 the next tick,
    //when the player updates its buildings or uses/deposists resources
    fn calc_ratios(
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
    pub fn generate(&mut self, all_buildings: &AllBuildings, tree: &DependencyTree) -> () {
        if self.gen.needs_update {
            self.gen.ratios = self.calc_ratios(tree, all_buildings);
            self.gen.make_gen_map(all_buildings, &self.buildings);
            self.gen.needs_update = false;
        }
        for (resource, amount) in self.gen.map.iter() {
            let mut crt = self.resources.entry(*resource).or_default();
            crt.current = crt.current.wrapping_add(*amount as u32);

            //checking if there enough resources for the next tick
            if *amount < 0 {
                match self.resources.get(resource) {
                    Some(x) if x.current < amount.abs() as u32 => self.gen.needs_update = true,
                    _ => (),
                };
            };
        }
    }
}

#[derive(Debug)]
pub struct Generator {
    needs_update: bool,
    map: GenMap,
    ratios: HashMap<BuildingID, f32>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            map: GenMap::new(),
            ratios: HashMap::new(),
            needs_update: true,
        }
    }
    pub fn make_gen_map(
        &mut self,
        all_buildings: &AllBuildings,
        player_buildings: &HashMap<BuildingID, OwnedBuilding>,
    ) -> () {
        let mut gen = GenMap::new();
        for (building, ratio) in self.ratios.iter() {
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
        self.map = gen;
        ()
    }
}

impl Default for Generator {
    fn default() -> Self {
        Generator::new()
    }
}
