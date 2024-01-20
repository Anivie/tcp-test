use std::mem::size_of;

use colored::Colorize;
use log::info;
use tokio::io::AsyncReadExt;
use tokio::sync::watch::Receiver;

use crate::GLOBAL_MAP;
use crate::raw_bindings::raw_bindings::{sendto, sockaddr, sockaddr_in};
use crate::tcp::packet::data::{Controller, ReceiveData};
use crate::tcp::packet::tcp_packet::TCPPacket;

impl Controller {
    pub async fn third_handshake_listener(&self, receiver: Receiver<Option<ReceiveData>>) {
        self.process_receiver(receiver, |receiver| unsafe {
            if let Some(value) = GLOBAL_MAP.read().get("enable_thrid-shaking") {
                if !value.value().downcast_ref::<bool>().is_some() {
                    return;
                }
            }else {
                return;
            }

            let receiver = &receiver.tcphdr.__bindgen_anon_1.__bindgen_anon_2;

            if receiver.syn() == 1 && receiver.ack() == 1 {
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
                GLOBAL_MAP.write().insert("enable_thrid-shaking", Box::new(false));
            }
        }).await;
    }

    pub async fn packet_printer(&self, receiver: Receiver<Option<ReceiveData>>) {
        self.process_receiver(receiver, |receiver| {
            let mut string = String::new();
            string.push_str(format!("Received packet with size{}: {{\n", receiver.packet_size).as_str());
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

    pub async fn fourth_handshake_listener(&self, receiver: Receiver<Option<ReceiveData>>) {
        self.process_receiver(receiver, |receiver| unsafe {
            if let Some(value) = GLOBAL_MAP.read().get("enable_fin-shaking") {
                if !value.value().downcast_ref::<bool>().is_some() {
                    return;
                }
            }else {
                return;
            }

            let receiver = &receiver.tcphdr.__bindgen_anon_1.__bindgen_anon_2;
            if receiver.fin() == 1 || receiver.ack() == 1 {
                info!("{}", "FIN-ACK handshake packet found, FIN-FINAL handshake packet being sent......".truecolor(200, 35, 55));
                let mut packet = TCPPacket::default::<_, String>(&self.address_to_remote, None, self.local_port).unwrap();

                let sent_size = sendto(
                    self.socket,
                    packet.fourth_handshake(
                        receiver.fin(),
                        receiver.ack_seq,
                        receiver.seq,
                    ),
                    packet.len(),
                    0,
                    &self.sockaddr_to_remote as *const sockaddr_in as *const sockaddr,
                    size_of::<sockaddr>() as u32
                );

                tracing::info!("fourth_handshake send: {}, with size: {}", packet, sent_size);
                // GLOBAL_MAP.write().insert("enable_fin-shaking", Box::new(false));
            }
        }).await;
    }
}