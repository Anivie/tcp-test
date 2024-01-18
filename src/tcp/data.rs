use std::ffi::c_int;
use std::ops::Deref;
use std::time::Duration;

use tokio::sync::watch::Receiver;
use tokio::time;

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
    pub data: String,
}

#[derive(Copy, Clone)]
pub struct Controller {
    pub socket: c_int,
    pub port: u16,
    pub sockaddr_to: sockaddr_in,
}

impl Controller {
    pub async fn process_receiver<F>(&self, mut receiver: Receiver<Option<ReceiveData>>, process: F)
        where
            F: Fn(&ReceiveData) -> (),
    {
        let mut intv = time::interval(Duration::from_millis(10));

        loop {
            if let Some(r) = receiver.borrow_and_update().deref() {
                process(r);
            }

            if receiver.changed().await.is_err() {
                break;
            }
            intv.tick().await;
        }
    }
}