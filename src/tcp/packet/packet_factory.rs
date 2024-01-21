use std::mem::size_of;

use crate::raw_bindings::raw_bindings::{sendto, sockaddr, sockaddr_in};
use crate::tcp::packet::data::Controller;
use crate::tcp::packet::tcp_packet::TCPPacket;
use crate::tcp::util::ChangingOrderSizes;

impl Controller {
    pub fn make_packet<T: Into<Vec<u8>>>(&self, data: Option<T>) -> TCPPacket {
        TCPPacket::default(&self.address_to_remote, data, self.local_port).unwrap()
    }

    pub fn make_packet_with_none(&self) -> TCPPacket {
        TCPPacket::default::<_, String>(&self.address_to_remote, None, self.local_port).unwrap()
    }

    pub fn send_packet(&self, tcppacket: &mut TCPPacket) -> isize {
        let sent_size = unsafe {
            sendto(
                self.socket,
                tcppacket.as_ptr(),
                tcppacket.len(),
                0,
                &self.sockaddr_to_remote as *const sockaddr_in as *const sockaddr,
                size_of::<sockaddr>() as u32
            )
        };

        sent_size
    }
}

impl TCPPacket {
    pub fn to_first_handshake(mut self) -> TCPPacket {
        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.set_syn(1);
        }
        self
    }

    pub fn to_third_handshake(mut self, response_ack_seq: u32, response_seq: u32) -> TCPPacket {
        unsafe {
            let mut tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;
            tcp_head.set_ack(1);
            tcp_head.seq = response_ack_seq;
            tcp_head.ack_seq = (response_seq.to_host() + 1).to_network();
        }

        self
    }

    pub fn to_fourth_handshake(mut self, response_fin: u16, response_ack_seq: u32, response_seq: u32) -> TCPPacket {
        unsafe {
            let mut tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;
            tcp_head.set_ack(1);

            tcp_head.seq = response_ack_seq;
            tcp_head.set_fin(response_fin);
            tcp_head.ack_seq = (response_seq.to_host() + 1).to_network();
        }

        self
    }

    pub fn to_reply_packet(mut self, response_seq: u32, response_ack: u32, data_size: u32) -> TCPPacket {
        let response_seq = response_seq.to_host();

        unsafe {
            let mut tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;

            tcp_head.set_ack(1);
            tcp_head.seq = response_ack;
            tcp_head.ack_seq = (response_seq + data_size).to_network();
        }

        self
    }

    pub fn to_fin_packet(mut self) -> TCPPacket {
        unsafe {
            let mut tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;

            tcp_head.set_fin(1);
            tcp_head.set_ack(1);
            tcp_head.ack_seq = 1.to_network();
        }

        self
    }
}
