use crate::buildings::BuildingID;
use crate::player::Username;
use crate::resources::ResourceID;
use crate::tile::{Position, Tile};
use crate::trade::Offer;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::error::Error;
use std::fmt;

/// A token used as a pass for a user
///
/// BEWARE! This is currently just a random u32 that offers
/// no crypto guarantees!
#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq)]
pub struct Token(u32);

impl Token {
    pub fn new() -> Token {
        //a very secure & unique token
        Token(3421545)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Response {
    Event(Event),
    Exception(Exception),
}

///Player-generated requests.
#[derive(Debug, Deserialize, Serialize)]
pub enum Event {
    Action(Action),
    World(World),
    //currently only supports simple trades
    Trade {
        from: Username,
        to: Username,
        offer: Offer,
    },
    Auth(Auth),
}

///Events linked to authentification
#[derive(Debug, Deserialize, Serialize)]
pub enum Auth {
    Login(Username, Token),
    Register(Username),
    NewToken(Token),
    Disconnect,
}

///Events affecting the world.
#[derive(Debug, Deserialize, Serialize)]
pub enum World {
    GetTile(Position),
    Tile(Tile),
}

///Events only affecting the player.
///
///All events that are made by a player AND only affect this player are Actions.
///Because these events will never need to be realyed by the host to the other servers
///it is not necessary to add the Username of the player.
///Instead the identification is done by though SocketAddr.
#[derive(Debug, Deserialize, Serialize)]
pub enum Action {
    ///Adding buildings to the player's empire. Refer to `Player::build`
    Build {
        pos: Position,
        building: BuildingID,
        amount: u32,
    },
    ///Remove buildings from the player's empire. Refer to `Player::demolish`
    Demolish {
        pos: Position,
        building: BuildingID,
        amount: u32,
    },
    ///Adding workers to the player's building. Refer to `Player::hire`
    Hire { building: BuildingID, amount: u32 },
    ///Remove workers from the player's building. Refer to `Player::fire`
    Fire { building: BuildingID, amount: u32 },
    ///Add resources to the player. Refer to `Player::deposit`
    Deposit { resource: ResourceID, amount: u32 },
    ///Remove resources from the player. Refer to `Player::withdraw`
    Withdraw { resource: ResourceID, amount: u32 },
}

///Errors resulting from Events.
#[derive(Debug, Deserialize, Serialize)]
pub enum Exception {
    PlaceHolder,
    //related to player Action
    InsufficientResource(ResourceID),
    InsufficientSlot(BuildingID),
    InsufficientStockpile(ResourceID),
    NotFound,

    //world exploration
    TileNotOwned(Position),

    //auth errors
    InvalidToken,
    Unregistered,
    AlreadyRegistered,
    LoggedOut,
}

impl fmt::Display for Exception {
    // add code here
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "This is a game error broadcasted thourgh Response::Expception."
        )
    }
}

impl Error for Exception {}
