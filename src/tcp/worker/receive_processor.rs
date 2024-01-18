use std::mem::size_of;
use std::ops::Deref;

use log::info;
use tokio::sync::watch::Receiver;

use crate::{REMOTE_ADDRESS, REMOTE_PORT};
use crate::raw_bindings::raw_bindings::{sendto, sockaddr, sockaddr_in};
use crate::tcp::data::{Controller, ReceiveData};
use crate::tcp::tcp_packet::TCPPacket;

impl Controller {
    pub async fn third_handshake(&self, mut receiver: Receiver<Option<ReceiveData>>) {
        loop {
            if let Some(receiver) = receiver.borrow_and_update().deref() {
                info!("Start printer loop{}", receiver);
                unsafe {
                    if receiver.tcphdr.__bindgen_anon_1.__bindgen_anon_2.syn() == 1 && receiver.tcphdr.__bindgen_anon_1.__bindgen_anon_2.ack() == 1 {
                        let address = format!("{}:{}", REMOTE_ADDRESS, REMOTE_PORT);
                        let mut packet = TCPPacket::default(address.as_str(), "data", self.port).unwrap();

                        let sent_size = sendto(
                            self.socket,
                            packet.first_handshake(),
                            packet.len(),
                            0,
                            &self.sockaddr_to as *const sockaddr_in as *const sockaddr,
                            size_of::<sockaddr>() as u32
                        );

                        info!("Send: {}, with size: {}", packet, sent_size);
                    }
                }
            }

            info!("sleeping~~~~~~~~");
            if receiver.changed().await.is_err() {
                println!("du!");
                break;
            }
            info!("loop!");
        }
    }




}