use std::mem::size_of;

use colored::Colorize;
use log::info;
use tokio::sync::watch::Receiver;

use crate::raw_bindings::raw_bindings::{sendto, sockaddr, sockaddr_in};
use crate::tcp::data::{Controller, ReceiveData};
use crate::tcp::tcp_packet::TCPPacket;

impl Controller {
    pub async fn third_handshake_listener(&self, receiver: Receiver<Option<ReceiveData>>) {
        self.process_receiver(receiver, |receiver| unsafe {
            let receiver = receiver.tcphdr.__bindgen_anon_1.__bindgen_anon_2;

            if receiver.syn().to_le() == 1 && receiver.ack().to_le() == 1 {
                info!("{}", "Secondary handshake packet found, tertiary handshake packet being sent......".truecolor(200, 35, 55));
                let mut packet = TCPPacket::default::<_, String>(&self.address_to_remote, None, self.local_port).unwrap();

                let sent_size = sendto(
                    self.socket,
                    packet.third_handshake(
                        receiver.ack_seq,
                        receiver.seq,
                    ),
                    packet.len(),
                    0,
                    &self.sockaddr_to_remote as *const sockaddr_in as *const sockaddr,
                    size_of::<sockaddr>() as u32
                );

                tracing::info!("third_handshake send: {}, with size: {}", packet, sent_size);
            }
        }).await;
    }

    pub async fn packet_printer(&self, receiver: Receiver<Option<ReceiveData>>) {
        self.process_receiver(receiver, |receiver| {
            let mut string = String::new();
            string.push_str("Received: {\n");
            string.push_str(format!("  received ip head: {}\n", receiver.iphdr).as_str());
            string.push_str(format!("  received tcp head: {}\n", receiver.tcphdr).as_str());
            string.push_str("}\n");
            tracing::info!("{}", string.truecolor(170, 170, 170));
        }).await;
    }

    pub async fn data_listener(&self, receiver: Receiver<Option<ReceiveData>>) {
        self.process_receiver(receiver, |receiver| {
            if let Some(data) = &receiver.data {
                let tmp = String::from_utf8_lossy(data);
                info!(
                    "Receive a string from {}: {}",
                    "server".truecolor(250, 108, 10),
                    tmp.replace("\r", "")
                    .replace("\n", "")
                    .truecolor(10, 163, 250)
                );
                let mut packet = TCPPacket::default::<_, String>(&self.address_to_remote, None, self.local_port).unwrap();

                let sent_size = unsafe {
                    sendto(
                        self.socket,
                        packet.reply_packet(
                            receiver.tcphdr.__bindgen_anon_1.__bindgen_anon_2.seq,
                            receiver.tcphdr.__bindgen_anon_1.__bindgen_anon_2.ack_seq,
                            data.len() as u32
                        ),
                        packet.len(),
                        0,
                        &self.sockaddr_to_remote as *const sockaddr_in as *const sockaddr,
                        size_of::<sockaddr>() as u32,
                    )
                };

                tracing::info!("data ack packet send: {}, with size: {}", packet, sent_size);
            }
        }).await;
    }
}