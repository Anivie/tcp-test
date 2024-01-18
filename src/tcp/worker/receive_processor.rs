use std::mem::size_of;

use log::info;
use tokio::sync::watch::Receiver;

use crate::{REMOTE_ADDRESS, REMOTE_PORT};
use crate::raw_bindings::raw_bindings::{sendto, sockaddr, sockaddr_in};
use crate::tcp::data::{Controller, ReceiveData};
use crate::tcp::tcp_packet::TCPPacket;

impl Controller {
    pub async fn third_handshake(&self, receiver: Receiver<Option<ReceiveData>>) {
        self.process_receiver(receiver, |receiver| unsafe {
            let receiver = receiver.tcphdr.__bindgen_anon_1.__bindgen_anon_2;

            if receiver.syn().to_le() == 1 && receiver.ack().to_le() == 1 {
                info!("发现二次握手包，正在发送三次握手包……");
                let address = format!("{}:{}", REMOTE_ADDRESS, REMOTE_PORT);
                let mut packet = TCPPacket::default::<_, String>(address, None, self.port).unwrap();

                let sent_size = sendto(
                    self.socket,
                    packet.third_handshake(
                        receiver.ack_seq,
                        receiver.seq,
                    ),
                    packet.len(),
                    0,
                    &self.sockaddr_to as *const sockaddr_in as *const sockaddr,
                    size_of::<sockaddr>() as u32
                );

                tracing::info!("third_handshake send: {}, with size: {}", packet, sent_size);
            }
        }).await;
    }
}