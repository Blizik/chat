#![allow(dead_code, unused_imports)]

use std::io::{self, prelude::*};

use mio::{*, net::{TcpListener, TcpStream}};
use slab::Slab;

extern crate chat;

use chat::{Handler, Peer, Message, MessageData, TrackerMessage};

fn main() {
    let mut tracker = Tracker::bind("0.0.0.0:1234".parse().unwrap()).unwrap();

    if let Err(e) = tracker.start() {
        println!("Error: {:?}", e);
    }
}

// The tracker server
#[derive(Debug)]
struct Tracker {
    connections: Slab<Peer>,
    listener: TcpListener,
    running: bool,
}

impl Handler for Tracker {
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
                Ok(0) => return Ok(
                    Message {
                        from,
                        data: MessageData::Disconnect,
                    }),
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

impl Tracker {
    fn bind(addr: std::net::SocketAddr) -> io::Result<Self> {
        let listener = TcpListener::bind(&addr)?;

        let connections = Slab::with_capacity(128);

        Ok(Self {
            connections,
            listener,
            running: false
        })
    }

    fn start(&mut self) -> io::Result<()> {
        self.running = true;

        let poll = Poll::new()?;
        let mut events = Events::with_capacity(1024);

        // ~~ Problem ~~
        // I want this to be a nice unique token number, I'd like to use -1 for
        // "self" but Token is a wrapper around usize. I mostly just want the
        // peer connections to have tokens matching their indices in the slab.

        // Also std::usize::MAX is an invalid token?
        const TRACKER: Token = Token(std::usize::MAX - 1);

        // This will eventually only be Ready::writable(), tracker only needs
        // to tell new clients where the peers are.
        poll.register(&self.listener, TRACKER, Ready::readable(), PollOpt::edge())?;

        while self.running {
            poll.poll(&mut events, None)?;

            for event in &events {
                match event.token() {
                    // This will be the only necessary branch for the tracker.
                    // Tracker only connects peers, does nothing with messages.
                    TRACKER => {
                        let token = self.accept()?;
                        let stream = &self.connections[token.0].stream;

                        poll.register(stream, token, Ready::readable(), PollOpt::edge())?;
                    }
                    Token(n) => {
                        let msg = self.recv(Token(n))?;
                        match msg.data { 
                            MessageData::Disconnect => self.drop(Token(n))?,
                            data => println!("{}: {:?}", n, data),
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn accept(&mut self) -> io::Result<Token> {
        let (stream, _) = self.listener.accept()?;

        let entry = self.connections.vacant_entry();
        let token = Token(entry.key());

        let peer = Peer {
            name: None,
            token,
            stream
        };

        entry.insert(peer);

        Ok(token)
    }

    fn drop(&mut self, peer: Token) -> io::Result<()> {
        println!("Dropping {}", peer.0);

        if self.connections.contains(peer.0) {
            self.connections.remove(peer.0);
        } else {
            let e = io::Error::new(
                io::ErrorKind::NotFound,
                "The peer with that token could not be found"
            );
            return Err(e);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tracker() {
        
    }
}
