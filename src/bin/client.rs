#![allow(dead_code, unused_imports)]

use std::io::{self, prelude::*};

use mio::{*, net::{TcpListener, TcpStream}};
use slab::Slab;

extern crate chat;

use chat::{Handler, Peer, Message, MessageData, TrackerMessage};

fn main() {
    let mut client = Client::connect("127.0.0.1:1234".parse().unwrap()).unwrap();

    client.send(Token(0), MessageData::PeerMessage {
        name: "Blizik".to_string(),
        msg: "Hello world!".to_string()
    }).unwrap();

    loop {}
}

// The state of the user's running program
#[derive(Debug)]
struct Client {
    connections: Slab<Peer>,
    listener: TcpListener,
    running: bool,
}


impl Handler for Client {
    fn send(&mut self, to: Token, data: MessageData) -> io::Result<usize> {
        let stream = &mut self.connections[to.0].stream;
        let json_data = serde_json::to_string(&data)?;

        stream.write(&json_data.as_bytes())
    }

    fn recv(&mut self, from: Token) -> io::Result<Message> {
        let stream = &mut self.connections[from.0].stream;

        let mut data = Vec::new();
        let mut buf = [0; 8];

        loop {
            match stream.read(&mut buf) {
                Ok(n) => data.extend_from_slice(&buf[..n]),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e)
            }
        }

        let json_data = std::str::from_utf8(&data).unwrap();
        let data = serde_json::from_str(&json_data)?;

        Ok(Message { from, data })
    }
}

impl Client {
    // Connect to a tracker
    fn connect(addr: std::net::SocketAddr) -> io::Result<Self> {
        let tracker = Peer {
            name: None,
            token: Token(0),
            stream: TcpStream::connect(&addr)?,
        };

        let listener = TcpListener::bind(&"0.0.0.0:8000".parse().unwrap())?;

        let mut connections = Slab::with_capacity(128);
        connections.insert(tracker);

        Ok(Self {
            connections,
            listener,
            running: false
        })
    }
}
