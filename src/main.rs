#![allow(dead_code, unused_imports)]

extern crate mio;

use std::io::prelude::*;

use mio::*;
use mio::net::{TcpListener, TcpStream};

fn main() {
    const SERVER: Token = Token(0);

    let addr = "127.0.0.1:13265".parse().unwrap();

    // Setup the server socket
    let server = TcpListener::bind(&addr).unwrap();

    // Create a poll instance
    let poll = Poll::new().unwrap();

    // Start listening for incoming connections
    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    // Create storage for incoming clients
    let mut streams = Vec::new();

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (stream, _) = server.accept().unwrap();

                    // Should be given a better token
                    let client_token = Token(streams.len() + 1);

                    poll.register(&stream, client_token, Ready::readable(), PollOpt::edge()).unwrap();

                    streams.push(stream);
                }
                Token(n) => {
                    #[cfg(all(unix, not(target_os = "fuchsia")))]
                    {
                        use mio::unix::{UnixReady};

                        // Client dropped
                        // https://carllerche.github.io/mio/mio/unix/struct.UnixReady.html#method.hup
                        if UnixReady::from(event.readiness()).is_hup() {
                            println!("Client {} dropped", n);
                        }
                    }
                    
                    let mut stream = &streams[n - 1];

                    // Very tiny, just to make sure the loop below works
                    let mut buf = [0; 8];
                    let mut out = Vec::new();

                    loop {
                        match stream.read(&mut buf) {
                            Ok(n) if n != 0 => out.extend_from_slice(&buf[..n]),
                            _ => break,
                        }
                    }
                    
                    if out.len() == 0 {
                        println!("Client {} dropped", n);
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
