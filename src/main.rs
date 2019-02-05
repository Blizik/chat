#![allow(dead_code, unused_imports)]

extern crate mio;

use std::io::prelude::*;

use mio::*;
use mio::net::{TcpListener, TcpStream};

fn main() {
    // Setup some tokens to allow us to identify which event is
    // for which socket.
    const SERVER: Token = Token(0);
    const CLIENT: Token = Token(1);

    let addr = "127.0.0.1:13265".parse().unwrap();

    // Setup the server socket
    let server = TcpListener::bind(&addr).unwrap();

    // Create a poll instance
    let poll = Poll::new().unwrap();

    // Start listening for incoming connections
    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    let mut streams = Vec::new();

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (stream, _) = server.accept().unwrap();

                    poll.register(&stream, Token(streams.len() + 1), Ready::readable(), PollOpt::edge()).unwrap();

                    streams.push(stream);
                }
                Token(n) => {
                    let mut stream = &streams[n - 1];

                    let mut buf = [0; 5];
                    let mut out = Vec::new();

                    loop {
                        match stream.read(&mut buf) {
                            Ok(n) if n != 0 => out.extend_from_slice(&buf[..n]),
                            _ => break,
                        }
                    }

                    let out = std::str::from_utf8(&out).unwrap().trim();
                    if out.len() > 0 {
                        println!("Client {}: {}", n, out);
                    }
                }
            }
        }
    }
}
