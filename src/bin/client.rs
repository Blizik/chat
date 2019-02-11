#![allow(dead_code, unused_imports)]

use std::io::{self, prelude::*};

use mio::{
    net::{TcpListener, TcpStream},
    *,
};
use slab::Slab;

extern crate chat;
use chat::{Handler, Message, MessageData, Peer, TrackerMessage};

const TRACKER: Token = Token(0);

fn main() {
    let mut client = Client::connect("127.0.0.1:1234".parse().unwrap()).unwrap();

    if let Err(e) = client.start() {
        println!("Error: {:?}", e);
    }
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
                Ok(0) => {
                    return Ok(Message {
                        from,
                        data: MessageData::Disconnect,
                    });
                }
                Ok(n) => data.extend_from_slice(&buf[..n]),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
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
            token: TRACKER,
            stream: TcpStream::connect(&addr)?,
        };

        let listener = TcpListener::bind(&"0.0.0.0:8000".parse().unwrap())?;

        let mut connections = Slab::with_capacity(128);
        connections.insert(tracker);

        Ok(Self {
            connections,
            listener,
            running: false,
        })
    }

    fn start(&mut self) -> io::Result<()> {
        self.running = true;

        let poll = Poll::new()?;
        let mut events = Events::with_capacity(1024);

        // Register to accept peers
        const LISTENER: Token = Token(std::usize::MAX - 1);
        poll.register(&self.listener, LISTENER, Ready::readable(), PollOpt::edge())?;

        // Register to recieve tracker messages
        poll.register(&self.connections[0].stream, TRACKER, Ready::readable(), PollOpt::edge())?;

        while self.running {
            poll.poll(&mut events, None)?;

            for event in &events {
                match event.token() {
                    LISTENER => { // New peer!
                        let token = self.accept()?;
                        let stream = &self.connections[token.0].stream;

                        poll.register(stream, token, Ready::readable(), PollOpt::edge())?;
                    }
                    TRACKER => { // Tracker is telling us something
                        let msg = self.recv(TRACKER)?;
                        match msg.data {
                            MessageData::Disconnect => {
                                println!("Connection to tracker lost, attempting to reestablish...");
                                println!("lol not actually");
                                std::process::exit(1);
                            }
                            MessageData::Tracker(msg) => {
                                use TrackerMessage::*;
                                match msg {
                                    Connect(peers) => println!("{:?}", peers),
                                    Broadcast(msg) => println!("{}", msg),
                                }
                            }
                            _ => {
                                println!("Tracker sent peer data.");
                                println!("(This should never happen)");
                                println!("{:?}", msg.data);
                            }
                        }
                    }
                    n => { // A peer sent a message
                        let msg = self.recv(n)?;
                        match msg.data {
                            MessageData::Disconnect => self.drop(n)?,
                            MessageData::Peer { name, msg } => {
                                println!("{}: {}", name, msg);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn accept(&mut self) -> io::Result<Token> {
        unimplemented!()
    }

    fn drop(&mut self, _peer: Token) -> io::Result<()> {
        unimplemented!()
    }
}
