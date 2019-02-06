#![allow(dead_code, unused_imports)]

extern crate mio;
extern crate slab;

use std::io::{self, prelude::*};

use mio::*;
use mio::net::{TcpListener, TcpStream};

use slab::Slab;

const LISTENER: Token = Token(0);

fn main() {
    let mut my_server = Server::new("127.0.0.1:8000").unwrap();
    my_server.run();
}

struct Server {
    listener: TcpListener,
    conns: Slab<TcpStream>
}

impl Server {
    fn new(addr: &str) -> io::Result<Self> {
        let addr = addr.parse().expect("Unable to parse addr string.");

        let listener = TcpListener::bind(&addr)?;
        let mut conns = Slab::with_capacity(128);

        conns.vacant_entry();

        Ok(Self {
            listener,
            conns,
        })
    }

    fn run(&mut self) {
        let poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(1024);

        poll.register(&self.listener, LISTENER, Ready::readable(), PollOpt::edge()).unwrap();

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in &events {
                println!("{:?}", event);
                match event.token() {
                    LISTENER => {
                        loop {
                            match self.listener.accept() {
                                Ok((socket, _)) => {
                                    let entry = self.conns.vacant_entry();
                                    // To allow server to have Token(0)
                                    let token = Token(entry.key() + 1);

                                    poll.register(&socket, token, Ready::readable(), PollOpt::edge()).unwrap();
                                    entry.insert(socket);
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                                Err(ref e) => panic!("{}", e)
                            }
                        }
                    }

                    Token(n) => {
                        // To allow server to have Token(0)
                        let n = n - 1;

                        let mut buf = [0; 8];
                        let mut out = Vec::new();

                        loop {
                            match self.conns.get(n).unwrap().read(&mut buf) {
                                Ok(0) => {
                                    self.conns.remove(n);
                                    break;
                                }
                                Ok(n) => {
                                    out.extend_from_slice(&buf[..n]);
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                                Err(ref e) => panic!("{}", e),
                            }
                        }

                        let out = std::str::from_utf8(&out).unwrap().trim();
                        println!("{}", out);
                    }
                }
            }
        }
    }
}