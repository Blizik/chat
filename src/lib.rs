#![allow(dead_code, unused_imports)]

pub extern crate mio;
pub extern crate slab;
pub extern crate serde;
pub extern crate serde_json;

use std::io::{self, prelude::*};

use mio::{*, net::{TcpListener, TcpStream}};
use slab::Slab;
use serde::{Deserialize, Serialize};

// Handle incoming and outgoing connections
pub trait Handler {
    // Encode to json string and send as bytes,
    // returning number of bytes written
    fn send(&mut self, to: Token, data: MessageData) -> io::Result<usize>;

    // Parse json string as Message struct and return the result
    fn recv(&mut self, from: Token) -> io::Result<Message>;
}

// Name: The username chosen by this peer
// Token: The index at which this peer appears in connections slab
// Stream: The actual stream upon which data is sent and recieved
#[derive(Debug)]
pub struct Peer {
    pub name: Option<String>,
    pub token: Token,
    pub stream: TcpStream,
}

#[derive(Debug)]
pub struct Message {
    pub from: Token,
    pub data: MessageData,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageData {
    Disconnect,
    TrackerMessage,
    PeerMessage {
        name: String,
        msg: String,
    },
}

#[derive(Debug)]
pub enum TrackerMessage {
    Connection(),
    Broadcast(String),
    Shutdown
}
