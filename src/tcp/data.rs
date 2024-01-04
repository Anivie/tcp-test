use std::ffi::CString;
use crate::raw_bindings::raw_bindings::{iphdr, tcphdr};

#[derive(Debug)]
pub struct PseudoHeader {
    pub source_address: u32,
    pub dest_address: u32,
    pub placeholder: u8,
    pub protocol: u8,
    pub tcp_length: u16,
}

pub struct DataGram {
    pub iphdr: iphdr,
    pub tcphdr: tcphdr,
    pub data: CString,
}