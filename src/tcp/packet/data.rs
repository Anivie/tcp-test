use std::ffi::c_int;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::raw_bindings::raw_bindings::{iphdr, sockaddr_in, tcphdr};

#[derive(Debug)]
pub struct PseudoHeader {
    pub source_address: u32,
    pub dest_address: u32,
    pub placeholder: u8,
    pub protocol: u8,
    pub tcp_length: u16,
}

#[derive(Default)]
pub struct ReceiveData {
    pub iphdr: iphdr,
    pub tcphdr: tcphdr,
    pub packet_size: usize,
    pub data: Option<Vec<u8>>,
}

#[derive(Clone)]
pub struct Controller {
    pub socket: c_int,
    pub local_port: u16,
    pub sockaddr_to_remote: sockaddr_in,
    pub address_to_remote: String,
    pub last_ack_number: Arc<RwLock<u32>>,
    pub last_seq_number: Arc<RwLock<u32>>
}