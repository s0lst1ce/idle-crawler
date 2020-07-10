use crate::buildings::{AllBuildings, Building, BuildingID, DependencyTree};
use crate::resources::ResourceID;
use crate::tile::Position;
use crate::tile::Tile;
use crate::trade::{Ledger, Offer, ResourceEntry};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Username = String;

/// Player stockpiles for a resource
///
/// Used to store the current holdings of a player over a specific resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct Stockpile {
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

/// Stats about a building type owned by a player
///
/// Holds data for a building type the player has.
/// That includes the total number of buildings of this type the player owns,
/// a mapping of where they are located and the workers stats for this building.
/// Having a single struct per building type instead of one per actual building
/// makes it simpler to build and demoslih the buildingi as well as managing employees.
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

/// Player's subjects
///
/// Holds data related to the population of a player's empire.
/// Most useful for dispatching citizens to jobs as well as managing the total population.
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
//however it shouldn't be saved in the JSON so it has to be saved eslewhere
//unless a field can be omitted by serde that is
pub type GenMap = HashMap<ResourceID, i32>;

/// Only interface to interact with an in-game player.
///
/// This is the most important struct of the game. It contains the player's empire
/// but also its trades, and generation maps. It is responsible for generating the player's
/// resources.
///
/// Most actions undertaken by the user will channel through this very struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    //HashMap<"building_name", OwnedBuilding>
    buildings: HashMap<BuildingID, OwnedBuilding>,
    people: Population,
    //HashMap<"resource_name", Stockpile>
    pub resources: HashMap<ResourceID, Stockpile>,
    //every other player the player is able to communicate with and the number of tiles they share.
    //The entry should be deleted when the count reaches 0
    contacts: HashMap<Username, u32>,
    //all tiles the player has built on are considered part of its territory. Thus tiles may be owned by more than 1 player.
    lands: Vec<Position>,
    trades: Ledger,
    #[serde(skip)]
    gen: Generator,
}

impl Player {
    ///Creates a new player. Later needs to be registered into GameData.
    pub fn new(buildings: &AllBuildings) -> Player {
        let mut p = Player {
            buildings: HashMap::new(),
            people: Population::new(),
            resources: HashMap::new(),
            contacts: HashMap::new(),
            lands: Vec::new(),
            trades: Ledger::new(),
            gen: Generator::new(),
        };
        //testing only
        p.add_building(
            Position { x: 0, y: 0 },
            BuildingID(0),
            buildings.get(&BuildingID(0)).unwrap(),
            1,
        );
        p.add_building(
            Position { x: 0, y: 0 },
            BuildingID(1),
            buildings.get(&BuildingID(1)).unwrap(),
            1,
        );
        p
    }

    //Returns the maximum amount of buildings of the type `id` the player can currrently build.
    pub fn max_buildable(&self, tiles: Vec<&Tile>, id: BuildingID, building: &Building) -> u32 {
        let mut max = u32::MAX;

        //if the building uses natural resources there must enough free slots
        //otherwise the player can't place the building
        if building.extractor {
            max = 0;
            for tile in tiles {
                if let Some(patch) = tile.resources.slots.get(&id) {
                    max += patch.total - patch.used
                }
            }
        }
        for (resource, qt) in building.construction_cost.iter() {
            if let Some(res) = self.resources.get(resource) {
                max = max.min(res.current / qt);
            } else {
                return 0;
            }
        }
        max
    }

    ///Attempts to build `amount` `building` in `tile`.
    ///Fails if `amount` is too big (see max_buildable)
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
        if amount > self.max_buildable(vec![tiles.1], id, building) {
            return Err(anyhow!(format!(
                "Can't build {:?} buildings of type ID{:?}, maximum is {:?}",
                amount,
                id,
                self.max_buildable(vec![tiles.1], id, building)
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
        (pos, tile): (&Position, &mut Tile),
        id: BuildingID,
        building: &Building,
        amount: u32,
    ) -> Result<()> {
        self.gen.needs_update = true;
        //making sure the player owns enough buildings
        if self.buildings.get(&id).unwrap().total < amount {
            return Err(anyhow!("Can't demolish more buildings than owned!"));
        };
        self.rm_building(pos, id, building, amount);
        tile.resources.slots.get_mut(&id).unwrap().used -= amount;
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
        let mut stock = self.resources.entry(id).or_default();
        if stock.maximum - stock.current < amount {
            Err(anyhow!(format!(
                "Can't add more resources than available space in {:?} stockpile!",
                id
            )))
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
            Err(anyhow!(format!(
                "Can't remove more resource ID{:?} than the player owns!",
                id
            )))
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
                        "Tried to hire more workers than available jobs for building ID{:?}!",
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

    /// Fires workers from a building type.
    ///
    /// Attempts to fire amount of workers from buildings of type id.
    ///
    /// # Errors
    /// This can fail for two reasons:
    /// - the player does not own the said building
    /// - the player tries to fire more workers than hired
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

    fn has_enough_for(&self, resources: &[ResourceEntry]) -> bool {
        for res_entry in resources {
            if let Some(res) = self.resources.get(&res_entry.id) {
                if res.current >= res_entry.amount {
                    continue;
                }
            }
            return false;
        }
        true
    }

    /// Creating a trade offer to a peer.
    ///
    /// Allows player to open a trade with another player.
    /// The method fails if the player does not have enough resources to garuantee the deal.
    ///
    // # Example
    //
    // ```
    //# use core::resources::ResourceID;
    //# use core::trade::{ResourceEntry, Offer};
    //# use core::Player;
    //# use core::buildings::load_buildings;
    //# let (all_buildings, _) = load_buildings();
    //# let mut player = Player::new(all_buildings);
    // let offer = Offer {offering: vec![ResourceEntry{id:ResourceID(0), amount: 12}], requesting: Vec::new()};
    // assert_eq!(true, player.open_trade("Toude".to_string(), offer).is_ok())
    //```
    pub fn open_trade(&mut self, with: Username, offer: Offer) -> Result<()> {
        if !self.has_enough_for(&offer.offering) {
            return Err(anyhow!("Not enough resources to comply with the Offer."));
        }
        for res_entry in &offer.offering {
            //we can unwrap because `has` made sure the key exists
            self.resources.get_mut(&res_entry.id).unwrap().current -= res_entry.amount;
        }
        self.trades.outbound.entry(with).or_default().push(offer);
        Ok(())
    }

    /// Removes a trade without handling Responses or other logic.
    ///
    /// # Errors
    /// Fails if the offer does not exists in offers.
    fn remove_offer(
        with: &Username,
        offer: &Offer,
        offers: &mut HashMap<Username, Vec<Offer>>,
    ) -> Result<()> {
        if let Some(contracts) = offers.get_mut(with) {
            for (idx, ongoing_offer) in contracts.iter().enumerate() {
                if ongoing_offer == offer {
                    contracts.remove(idx);
                    return Ok(());
                }
            }
        }
        Err(anyhow!("No such trades with this player."))
    }

    /// Cancel an active trade opened by the player.
    ///
    /// Let's the player who made the trade cancel their offer.
    /// This restores the "reserved" resources. Excesses are disposed of.
    /// This may receive an Exception Response because the host server may have registered
    /// a refusal of agreement for the said trade.
    pub fn cancel_trade(&mut self, with: &Username, offer: &Offer) -> Result<()> {
        //first we make sure the Offer is still valid (ie: exists).
        Player::remove_offer(with, offer, &mut self.trades.outbound)?;
        Ok(())
    }

    /// Accept a received trade offer
    ///
    /// Attempts to accept a trade offer made by a foreign party.
    ///
    /// # Errors
    /// This fails if:
    /// - the trade cancellation was registered before the agreement.
    /// - the player does not have enough resources to complete fulfill the offer
    pub fn accept_trade(&mut self, with: &Username, offer: &Offer) -> Result<()> {
        if self.has_enough_for(&offer.requesting) {
            Player::remove_offer(with, offer, &mut self.trades.inbound)?;
            return Ok(());
        }
        Err(anyhow!(
            "Player does not have enough resources to fulfill the offer."
        ))
    }

    /// Computes resources generations optimal ratios.
    ///
    //Creates a map with the lowest factor at which the building can work.
    //The key is the building and the value its current efficiency.
    //The later is calculated from available resources
    //and takes the production chain into account
    //It needs to be ran everytime a resource's stockpile would reach 0 the next tick,
    //when the player updates its buildings or uses/deposists resources
    //WARN: I think I forgot to take the prod of the other buildings into account ^^
    //WARN: It doesn't have a high impact but may reduce perfs on low resources.
    fn calc_ratios(
        &self,
        tree: &DependencyTree,
        all_buildings: &AllBuildings,
    ) -> HashMap<BuildingID, f32> {
        let mut ratios: HashMap<BuildingID, f32> = HashMap::new();
        println!("Tree: {:?}", tree);
        //We set buildings which don't consume resources to their optimal efficiency.
        for building_id in tree.1.iter() {
            ratios.insert(*building_id, 1.0_f32);
        }

        //We calculate the maximal efficiency of all buildings which consume resources.
        for (resource, depends) in tree.0.iter() {
            //the total amount of `resource` needed by the player's empire
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
        };
        for (resource, amount) in self.gen.map.iter() {
            let mut crt = self.resources.entry(*resource).or_default();
            crt.current = crt.current.wrapping_add(*amount as u32).min(crt.maximum);

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

/// Player  resource generator
///
/// This can be conceived as a cache to speed up calculations.
/// With it is not necessary to calculate the ratios each tick, speeding up the update.
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
        for (building, _) in player_buildings.iter() {
            //adding resources produced per tick
            for (resource, amount) in all_buildings.get(building).unwrap().produced.iter() {
                *gen.entry(*resource).or_insert(0) +=
                    ((*amount * player_buildings.get(building).unwrap().workers.0) as f32
                        * self.ratios.get(building).unwrap()) as i32;
            }
            //substracting resources consumed per tick
            for (resource, amount) in all_buildings.get(building).unwrap().consumed.iter() {
                *gen.entry(*resource).or_insert(0) -=
                    ((*amount * player_buildings.get(building).unwrap().workers.0) as f32
                        * self.ratios.get(building).unwrap()) as i32;
            }
        }

        self.map = gen
    }
}

impl Default for Generator {
    fn default() -> Self {
        Generator::new()
    }
}
