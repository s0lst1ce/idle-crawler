use serde_json::Value;
use serde_json;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::{env, io};
use tokio;
use tokio::net::UdpSocket;

//tile position
#[derive(Debug, Serialize, Deserialize)]
struct Position {
    x: i32,
    y: i32,
}

struct Server {
    socket: UdpSocket,
    clients: HashMap<SocketAddr, Client>,
    buf: Vec<u8>,
}

struct Client {
    //None if the user hasn't been authentificated
    username: Option<String>,
    //the tiles for which information has to be sent
    watching: Vec<Position>,

    //buffer and uptil where it must be read
    pending: Option<Value>,
}


impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            mut socket,
            mut clients,
            mut buf,
        } = self;

        loop {
            //writing the data when available
            let (read_to, client_addr) = socket.recv_from(&mut buf).await?;
            //making sure the data is valid json
            let data: Value = serde_json::from_slice(&buf[..read_to])?;
            println!("{:?}", data);
            clients.entry(client_addr).or_insert(Client::new()).pending = Some(data);
        }
    }
}

impl Client {
    fn new() -> Client {
        Client {
            username: None,
            watching: vec![],
            pending: None
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

    let server = Server {
        socket,
        clients: HashMap::new(),
        buf: vec![0; 1024],
    };

    // This starts the server task.
    server.run().await?;

    Ok(())
}