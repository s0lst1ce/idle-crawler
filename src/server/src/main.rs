use anyhow::Result;
use core::response::{Action, Auth, Event, Exception, Response, Token};
use core::Username;
use core::{BuildingID, Game, ResourceID};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fs::write;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::{env, io};
use tokio;
use tokio::net::UdpSocket;

//How should I determine the size of the buffer? By calculating the size of the largest event (Build)
const BUFFER_SIZE: usize = 1024;
const USERS_PATH: &str = "accounts.json";

type Accounts = HashMap<Username, Token>;

struct Server {
    game: (Sender<(Username, Event)>, Receiver<(Username, Exception)>),
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
            Response::Auth(auth) => match auth {
                Auth::Disconnect => if self.clients.remove(&addr).is_none() {
                        self.socket
                            .send_to(
                                serde_json::to_string(&Exception::LoggedOut)
                                    .unwrap()
                                    .as_bytes(),
                                addr,
                            )
                            .await?;} else {
                                self.dispatch(&Response::Exception(Exception::LoggedOut), addr).await?;
                            }
                Auth::Login(username, token) => self.login(addr, username, &token).await?,
                Auth::Register(username) => self.register(addr, username).await?,
                Auth::NewToken(_) => panic!("The server should never receive NewToken! It is supposed to send it when registration succeeds."),
            }
            Response::Event(event) => {
                //we check for auth first because all other events require a logged user
                if self.clients.contains_key(&addr){
                    match self.clients.get(&addr).unwrap().clone() {
                        Some(username) => self.send_to_game(username, event).await?,
                        None => self.dispatch(&Response::Exception(Exception::LoggedOut), addr).await?
                    }
                }
            }
            Response::Exception(_) => (),
        }
        //we should only have at most one exception waiting here.
        if let Ok(exception) = self.game.1.try_recv() {
            let crt_addr = *self
                .clients
                .iter()
                .find(|user| {
                    if let Some(username) = user.1 {
                        *username == exception.0
                    } else {
                        false
                    }
                })
                .unwrap()
                .0;
            self.dispatch(&Response::Exception(exception.1), crt_addr)
                .await?;
        }
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        loop {
            self.update_once().await?;
        }
    }

    async fn send_to_game(&mut self, username: Username, event: Event) -> Result<(), io::Error> {
        if let Err(_) = self.game.0.send((username, event)) {
            panic!("Something broke the channel between the game and main threads!");
        }
        Ok(())
    }

    async fn dispatch(&mut self, response: &Response, addr: SocketAddr) -> Result<(), io::Error> {
        self.socket
            .send_to(serde_json::to_string(response).unwrap().as_bytes(), addr)
            .await?;
        //this is for dev only!
        save_accounts(USERS_PATH, &self.accounts).await;
        Ok(())
    }

    async fn register(&mut self, addr: SocketAddr, username: Username) -> Result<(), io::Error> {
        if self.accounts.contains_key(&username) {
            self.dispatch(&Response::Exception(Exception::AlreadyRegistered), addr)
                .await?;
        }
        let token = Token::new();
        self.dispatch(&Response::Auth(Auth::NewToken(token)), addr)
            .await?;
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
                    self.dispatch(&Response::Exception(Exception::InvalidToken), addr)
                        .await?;
                }
            }
            None => {
                self.dispatch(&Response::Exception(Exception::Unregistered), addr)
                    .await?
            }
        }
        Ok(())
    }
}

async fn load_accounts<A: AsRef<Path>>(path: A) -> Accounts {
    let file = std::fs::read(path).expect("coudln't read accounts file");
    serde_json::from_slice(&file).expect("couldn't serialize accounts JSON")
}

async fn save_accounts<A: AsRef<Path>>(path: A, accounts: &Accounts) -> Result<()> {
    Ok(write(path, serde_json::to_string(accounts)?)?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:".to_owned() + &env::args().nth(1).unwrap_or_else(|| "6142".to_string());

    let socket = UdpSocket::bind(&addr).await?;

    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();
    let server = Server {
        game: (tx1, rx2),
        accounts: load_accounts(USERS_PATH).await,
        socket,
        clients: HashMap::new(),
        buf: vec![0; BUFFER_SIZE],
    };
    println!("Listening on: {}", server.socket.local_addr()?);

    //this is a DEV ONLY section that will need re-work
    let mut game = Game::new(0);
    println!(
        "Resources: {:?}\nBuildings: {:?}",
        game.get_resources(),
        game.get_buildings()
    );
    let p = game.add_player(String::from("Toude"))?;
    println!(
        "An event in JSON:\n {:?}\n",
        serde_json::to_string(&Response::Event(Event::Player(Action::Hire {
            building: BuildingID(0),
            amount: 3
        })))
    );
    p.deposit(ResourceID(0), 30)?;
    p.hire(BuildingID(0), 2)?;
    p.hire(BuildingID(1), 1)?;
    println!("Toude {:?}", p);
    thread::spawn(move || game.run(1, tx2, rx1));
    server.run().await?;

    //running server

    Ok(())
}
