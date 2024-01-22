use colored::Colorize;
use log::info;
use tokio::sync::watch::Receiver;

use crate::tcp::packet::data::{Controller, ReceiveData, SpacilProcessor};

impl Controller {
    pub async fn third_handshake_listener(&self, receiver: Receiver<Option<ReceiveData>>) {
        processor!(self, receiver, SpacilProcessor::InitHandshake, |receiver| {

            if receiver.tcphdr.syn() == 1 && receiver.tcphdr.ack() == 1 {
                info!("{}", "Secondary handshake packet found, tertiary handshake packet being sent......".truecolor(200, 35, 55));
                let mut packet = self.make_packet_with_none().to_third_handshake(receiver.tcphdr.ack_seq, receiver.tcphdr.seq);

                let sent_size = self.send_packet(&mut packet);

                info!("third_handshake send: {}, with size: {}", packet, sent_size);
                *self.spacil.write() = SpacilProcessor::None;
            }
        });
    }

    pub async fn packet_printer(&self, receiver: Receiver<Option<ReceiveData>>) {
        processor!(self, receiver, SpacilProcessor::None, |receiver| {
            let mut string = String::new();
            string.push_str(format!("Received packet with size {}: {{\n", receiver.packet_size).as_str());
            string.push_str(format!("  received ip head: {}\n", receiver.iphdr).as_str());
            string.push_str(format!("  received tcp head: {}\n", receiver.tcphdr).as_str());
            string.push_str("}\n");
            tracing::info!("{}", string.truecolor(170, 170, 170));
        })
    }

    pub async fn data_listener(&self, receiver: Receiver<Option<ReceiveData>>) {
        processor!(self, receiver, SpacilProcessor::None, |receiver| {
            if let Some(data) = &receiver.data {
                let tmp = String::from_utf8_lossy(data);
                info!(
                    "Receive a string from {}: {}",
                    "server".truecolor(250, 108, 10),
                    tmp.replace("\r", "")
                    .replace("\n", "")
                    .truecolor(10, 163, 250)
                );
                let mut packet = self
                                                .make_packet_with_none()
                                                .to_data_ack_packet(receiver.tcphdr.seq, receiver.tcphdr.ack_seq, data.len() as u32);

                let sent_size = self.send_packet(&mut packet);

                tracing::info!("data ack packet send: {}, with size: {}", packet, sent_size);
            }
        })
    }

    pub async fn fourth_handshake_listener(&self, receiver: Receiver<Option<ReceiveData>>) {
        processor!(self, receiver, SpacilProcessor::WaveHandshake, |receiver| {
            if receiver.tcphdr.fin() == 1 && receiver.tcphdr.ack() == 1 {
                info!("{}", "FIN-ACK handshake packet found, FIN-FINAL handshake packet being sent......".truecolor(200, 35, 55));
                let mut packet = self.make_packet_with_none().to_fourth_handshake(receiver.tcphdr.ack_seq, receiver.tcphdr.seq);

                let sent_size = self.send_packet(&mut packet);

                tracing::info!("fourth_handshake send: {}, with size: {}", packet, sent_size);
                *self.spacil.write() = SpacilProcessor::None;
                info!("FIN-ACK success, bye, my dear baby~");
                std::process::exit(0);
            }
        });
    }
}