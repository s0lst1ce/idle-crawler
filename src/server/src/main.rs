use anyhow::Result;
use core::response::{Action, Auth, Event, Exception, Response, Token};
use core::Username;
use core::{BuildingID, Game, Position, ResourceID};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::path::Path;
use std::thread;
use std::{env, io};
use tokio;
use tokio::net::lookup_host;
use tokio::net::UdpSocket;

//How should I determine the size of the buffer? By calculating the size of the largest event (Build)
const BUFFER_SIZE: usize = 1024;
const USERS_PATH: &str = "accounts.json";

type Accounts = HashMap<Username, Token>;

struct Server {
    accounts: Accounts,
    socket: UdpSocket,
    //None means not authentificated
    clients: HashMap<SocketAddr, Option<Username>>,
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
        match response {
            Response::Event(event) => {
                //we check for auth first because all other events require a logged user
                if let Event::Auth(auth) = event {
                    match auth {
                        Auth::Disconnect => if self.clients.remove(&addr).is_none() {
                                self.socket
                                    .send_to(
                                        serde_json::to_string(&Exception::LoggedOut)
                                            .unwrap()
                                            .as_bytes(),
                                        addr,
                                    )
                                    .await?;} else {
                                        self.dispatch(&Response::Exception(Exception::LoggedOut), addr);
                                    }
                        Auth::Login(username, token) => self.login(addr, username, &token).await?,
                        Auth::Register(username) => self.register(addr, username).await?,
                        Auth::NewToken(_) => panic!("The server should never receive NewToken! It is supposed to send it when registration succeeds."),
                    }
                } else {
                    if self.clients.entry(addr).or_default().is_some() {
                        match event {
                            Event::Action(action) => unimplemented!(),
                            Event::World(world) => unimplemented!(),
                            Event::Trade { from, to, offer } => unimplemented!(),
                            Event::Auth(_) => panic!("Event::Auth shouldn't be matched now!"),
                        }
                    } else {
                        self.dispatch(&Response::Exception(Exception::LoggedOut), addr).await?;
                    }
                }
            }
            Response::Exception(exception) => (),
        }
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        loop {
            self.update_once().await?;
        }
    }

    async fn dispatch(&mut self, response: &Response, addr: SocketAddr) -> Result<(), io::Error> {
        self.socket.send_to(serde_json::to_string(response).unwrap().as_bytes(), addr).await?;
        Ok(())
    }

    async fn register(&mut self, addr: SocketAddr, username: Username) -> Result<(), io::Error> {
        if self.accounts.contains_key(&username) {
            self.dispatch(&Response::Exception(Exception::AlreadyRegistered), addr);
        }
        let token = Token::new();
        self.dispatch(&Response::Event(Event::Auth(Auth::NewToken(token))), addr).await?;
        Ok(())
    }

    async fn login(
        &mut self,
        addr: SocketAddr,
        username: Username,
        token: &Token,
    ) -> Result<(), io::Error> {
        match self.accounts.get(&username) {
            Some(r_token) => {
                if r_token == token {
                    *self.clients.get_mut(&addr).unwrap() = Some(username)
                } else {
                    self.dispatch(&Response::Exception(Exception::InvalidToken), addr).await?;
                }
            }
            None => self.dispatch(&Response::Exception(Exception::Unregistered), addr).await?,
        }
        Ok(())
    }
}

fn load_accounts<A: AsRef<Path>>(path: A) -> Accounts {
    let file = std::fs::read(path).expect("coudln't read accounts file");
    serde_json::from_slice(&file).expect("couldn't serialize accounts JSON")
}

fn save_acounts<A: AsRef<Path>>(path: A, accounts: Accounts) -> Result<()> {
    unimplemented!()
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
        accounts: load_accounts(USERS_PATH),
        socket,
        clients: HashMap::new(),
        buf: vec![0; BUFFER_SIZE],
    };

    //running server
    server.run().await?;

    Ok(())
}
