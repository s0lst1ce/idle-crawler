use core::Username;
use std::thread;
use anyhow::Result;
use core::response::{Action, Event, Exception, Response};
use core::{BuildingID, Game, Position, ResourceID};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::{env, io};
use tokio;
use tokio::net::lookup_host;
use tokio::net::UdpSocket;

//How should I determine the size of the buffer? By calculating the size of the largest event (Build)
const BUFFER_SIZE: usize = 1024;

pub struct Client {
    //None if the user hasn't been authentificated
    username: Option<Username>,
    //the tiles for which information has to be sent
    watching: Vec<Position>,
}

impl Client {
    fn new() -> Client {
        Client {
            username: None,
            watching: vec![],
        }
    }
}

struct Server {
    socket: UdpSocket,
    clients: HashMap<SocketAddr, Client>,
    buf: Vec<u8>,
}

impl Server {
    async fn poll(&mut self) -> Result<(SocketAddr, Response), io::Error> {
        //writing the data when available
        let (read_to, client_addr) = self.socket.recv_from(&mut self.buf).await?;
        //making sure the data is valid json
        let response: Response = serde_json::from_slice(&self.buf[..read_to])?;
        println!("Received JSON: {:?}", response);
        //self.clients.entry(client_addr).or_insert(Client::new()).pending = Some(reponse);
        Ok((client_addr, response))
    }
    async fn update_once(&mut self) -> Result<(), io::Error> {
        let (addr, response) = self.poll().await?;
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        loop {
            self.update_once().await?;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening on: {}", socket.local_addr()?);

    //this is a DEV ONLY section that will need re-work
    let mut game = Game::new(0);
    println!(
        "Resources: {:?}\nBuildings: {:?}",
        game.get_resources(),
        game.get_buildings()
    );
    let p = game.add_player("Toude".to_string())?;
    println!(
        "An event in JSON:\n {:?}\n",
        serde_json::to_string(&Response::Event(Event::Action(Action::Hire {
            building: BuildingID(0),
            amount: 3
        })))
    );
    p.deposit(ResourceID(0), 30)?;
    p.hire(BuildingID(0), 2)?;
    p.hire(BuildingID(1), 1)?;
    println!("Toude {:?}", p);
    thread::spawn(move || game.run(1));

    let server = Server {
        socket,
        clients: HashMap::new(),
        buf: vec![0; BUFFER_SIZE],
    };

    //running server
    server.run().await?;

    Ok(())
}
