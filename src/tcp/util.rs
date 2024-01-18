use std::fmt::{Display, Formatter};

use crate::raw_bindings::raw_bindings::{htonl, htons, ntohl, ntohs};
use crate::tcp::data::ReceiveData;

pub trait ToAddress {
    fn to_address(&self) -> Option<(u16, &str)>;
}

impl<T: AsRef<str>> ToAddress for T {
    fn to_address(&self) -> Option<(u16, &str)> {
        let parts: Vec<&str> = self.as_ref().split(':').collect();
        if parts.is_empty() { return None; }
        if parts.len() == 1 { return None; }

        let addr = parts[0];
        let port = match parts[1].parse::<u16>() {
            Ok(p) => { p }
            Err(_) => { return None }
        };
        Some((port, addr))
    }
}

impl Display for ReceiveData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\nIP head: {},\n TCP head: {},\n Data: {}\n]}}",
            self.iphdr,
            self.tcphdr,
            self.data,
        )
    }
}

pub trait ChangingOrderSizes<T> {
    #[inline]
    fn to_network(self) -> T;
    #[inline]
    fn to_host(self) -> T;
}

impl ChangingOrderSizes<u16> for u16{
    fn to_network(self) -> u16 {
        unsafe {
            htons(self)
        }
    }

    fn to_host(self) -> u16 {
        unsafe {
            ntohs(self)
        }
    }
}

impl ChangingOrderSizes<u32> for u32{
    fn to_network(self) -> u32 {
        unsafe {
            htonl(self)
        }
    }

    fn to_host(self) -> u32 {
        unsafe {
            ntohl(self)
        }
    }
}