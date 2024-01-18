use std::ffi::c_int;

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
    pub data: Option<Vec<u8>>,
}

#[derive(Copy, Clone)]
pub struct Controller {
    pub socket: c_int,
    pub port: u16,
    pub sockaddr_to: sockaddr_in,
}