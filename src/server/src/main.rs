pub mod response;
use std::time::Duration;
use tokio::select;
use std::thread;
use anyhow::{Result};
use serde_json::Value;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::{env, io};
use tokio;
use tokio::net::UdpSocket;
use server::{Position, Game, clock};


const BUFFER_SIZE: usize = 1024;


pub struct Client {
    //None if the user hasn't been authentificated
    username: Option<String>,
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
    async fn poll(&mut self) -> Result<(), io::Error> {
        //writing the data when available
        let (read_to, client_addr) = self.socket.recv_from(&mut self.buf).await?;
        //making sure the data is valid json
        let data: Value = serde_json::from_slice(&self.buf[..read_to])?;
        println!("Received JSON: {:?}", data);
        //self.clients.entry(client_addr).or_insert(Client::new()).pending = Some(data);
        Ok(())
    }
    async fn update_once(&mut self) -> Result<(), io::Error> {
        self.poll().await?;
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
    println!("Resources: {:?}\nBuildings: {:?}", game.get_resources(), game.get_buildings());
    let p = game.add_player("Toude".to_string())?;
    p.deposit(0, 100)?;
    thread::spawn(move || {
        let mut clock = clock::Clock::new(30);
        loop {
            game.update();
            thread::sleep(clock.tick()); //the specific time should be handled by a clock
            println!("Players {:?}", game.get_players());
    }});


    let server = Server {
        socket,
        clients: HashMap::new(),
        buf: vec![0; BUFFER_SIZE],
    };

    //running server
    server.run().await?;

    Ok(())
}