use serde_json::Value;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::{env, io};
use tokio;
use tokio::net::UdpSocket;
use server::{Client, Game};


const BUFFER_SIZE: usize = 1024;

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
        println!("{:?}", data);
        //self.clients.entry(client_addr).or_insert(Client::new()).pending = Some(data);
        Ok(())
    }
    async fn update_once(&mut self) -> Result<(), io::Error> {
        self.poll().await?;
        Ok(())
    }

    async fn run(mut self) -> Result<(), io::Error> {
        loop {
            self.update_once().await?
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

    let mut server = Server {
        socket,
        clients: HashMap::new(),
        buf: vec![0; BUFFER_SIZE],
    };

    //running server
    server.run().await?;

    Ok(())
}