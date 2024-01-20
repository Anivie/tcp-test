use std::ffi::c_void;

use crate::tcp::packet::tcp_packet::TCPPacket;
use crate::tcp::util::ChangingOrderSizes;

impl TCPPacket {
    pub fn first_handshake(&mut self) -> *const c_void {
        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.set_syn(1);
        }
        self.as_ptr()
    }

    pub fn third_handshake(&mut self, response_ack_seq: u32, response_seq: u32) -> *const c_void {
        unsafe {
            let mut tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;
            tcp_head.set_ack(1);
            tcp_head.seq = response_ack_seq;
            tcp_head.ack_seq = (response_seq.to_host() + 1).to_network();
        }

        self.as_ptr()
    }

    pub fn fourth_handshake(&mut self, response_fin: u16, response_ack_seq: u32, response_seq: u32) -> *const c_void {
        unsafe {
            let mut tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;
            tcp_head.set_ack(1);

            tcp_head.seq = response_ack_seq;
            tcp_head.set_fin(response_fin);
            tcp_head.ack_seq = (response_seq.to_host() + 1).to_network();
        }

        self.as_ptr()
    }

    pub fn reply_packet(&mut self, response_seq: u32, response_ack: u32, data_size: u32) -> *const c_void {
        let response_seq = response_seq.to_host();

        unsafe {
            let mut tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;

            tcp_head.set_ack(1);
            tcp_head.seq = response_ack;
            tcp_head.ack_seq = (response_seq + data_size).to_network();
        }

        self.as_ptr()
    }

    pub fn fin_packet(&mut self) -> *const c_void {
        unsafe {
            let mut tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;

            tcp_head.set_fin(1);
            tcp_head.set_ack(1);
            tcp_head.ack_seq = 1.to_network();
        }

        self.as_ptr()
    }
}
