use std::mem::size_of;

use crate::raw_bindings::raw_bindings::{sendto, sockaddr, sockaddr_in};
use crate::tcp::packet::data::{Controller, SpacilProcessor};
use crate::tcp::packet::tcp_packet::TCPPacket;
use crate::tcp::util::ChangingOrderSizes;

/// Controller struct implementation
impl Controller {
    /// Creates a TCP packet with data
    ///
    /// # Arguments
    ///
    /// * `data` - A generic type that can be converted into a Vec<u8>
    ///
    /// # Returns
    ///
    /// * `TCPPacket` - The created TCP packet
    #[inline]
    pub fn make_packet_with_data<T: Into<Vec<u8>>>(&self, data: T) -> TCPPacket {
        TCPPacket::default(&self.address_to_remote, Some(data), self.local_port).unwrap()
    }

    /// Creates a TCP packet without data
    ///
    /// # Returns
    ///
    /// * `TCPPacket` - The created TCP packet
    #[inline]
    pub fn make_packet_with_none(&self) -> TCPPacket {
        TCPPacket::default::<_, String>(&self.address_to_remote, None, self.local_port).unwrap()
    }

    /// Sends a TCP packet with a SpacilProcessor
    ///
    /// # Arguments
    ///
    /// * `tcppacket` - A mutable reference to the TCP packet to be sent
    /// * `spacial` - The SpacilProcessor to be used
    ///
    /// # Returns
    ///
    /// * `isize` - The size of the sent packet
    #[inline]
    pub fn send_packet_spacial(&self, tcppacket: &mut TCPPacket, spacial: SpacilProcessor) -> isize {
        *self.spacil.write() = spacial;
        self.send_packet(tcppacket)
    }

    /// Sends a TCP packet
    ///
    /// # Arguments
    ///
    /// * `tcppacket` - A mutable reference to the TCP packet to be sent
    ///
    /// # Returns
    ///
    /// * `isize` - The size of the sent packet
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

/// TCPPacket struct implementation
impl TCPPacket {
    /// Converts the packet to a first handshake packet
    ///
    /// # Returns
    ///
    /// * `TCPPacket` - The converted TCP packet
    pub fn to_first_handshake(mut self) -> TCPPacket {
        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.set_syn(1);
        }
        self
    }

    /// Converts the packet to a third handshake packet
    ///
    /// # Arguments
    ///
    /// * `response_ack_seq` - The response acknowledgement sequence
    /// * `response_seq` - The response sequence
    ///
    /// # Returns
    ///
    /// * `TCPPacket` - The converted TCP packet
    pub fn to_third_handshake(mut self, response_ack_seq: u32, response_seq: u32) -> TCPPacket {
        unsafe {
            let tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;
            tcp_head.set_ack(1);
            tcp_head.seq = response_ack_seq;
            tcp_head.ack_seq = (response_seq.to_host() + 1).to_network();
        }

        self
    }

    /// Converts the packet to a fourth handshake packet
    ///
    /// # Arguments
    ///
    /// * `response_ack_seq` - The response acknowledgement sequence
    /// * `response_seq` - The response sequence
    ///
    /// # Returns
    ///
    /// * `TCPPacket` - The converted TCP packet
    pub fn to_fourth_handshake(mut self, response_ack_seq: u32, response_seq: u32) -> TCPPacket {
        unsafe {
            let tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;
            tcp_head.set_ack(1);

            tcp_head.seq = (response_ack_seq.to_host() + 1).to_network();
            tcp_head.ack_seq = (response_seq.to_host() + 1).to_network();
        }

        self
    }

    /// Converts the packet to a data acknowledgement packet
    ///
    /// # Arguments
    ///
    /// * `response_seq` - The response sequence
    /// * `response_ack` - The response acknowledgement
    /// * `data_size` - The size of the data
    ///
    /// # Returns
    ///
    /// * `TCPPacket` - The converted TCP packet
    pub fn to_data_ack_packet(mut self, response_seq: u32, response_ack: u32, data_size: u32) -> TCPPacket {
        let response_seq = response_seq.to_host();

        unsafe {
            let tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;

            tcp_head.set_ack(1);
            tcp_head.seq = response_ack;
            tcp_head.ack_seq = (response_seq + data_size).to_network();
        }

        self
    }

    /// Converts the packet to a data packet
    ///
    /// # Arguments
    ///
    /// * `response_seq` - The response sequence
    /// * `response_ack` - The response acknowledgement
    ///
    /// # Returns
    ///
    /// * `TCPPacket` - The converted TCP packet
    pub fn to_data_packet(mut self, response_seq: u32, response_ack: u32) -> TCPPacket {
        unsafe {
            let tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;

            tcp_head.set_psh(1);
            tcp_head.set_ack(1);

            tcp_head.seq = response_ack;
            tcp_head.ack_seq = response_seq;
        }

        self
    }

    /// Converts the packet to a FIN packet
    ///
    /// # Arguments
    ///
    /// * `response_seq` - The response sequence
    /// * `response_ack` - The response acknowledgement
    ///
    /// # Returns
    ///
    /// * `TCPPacket` - The converted TCP packet
    pub fn to_fin_packet(mut self, response_seq: u32, response_ack: u32) -> TCPPacket {
        unsafe {
            let tcp_head = &mut self.tcp_head.__bindgen_anon_1.__bindgen_anon_2;

            tcp_head.set_fin(1);
            tcp_head.set_ack(1);

            tcp_head.ack_seq = response_seq;
            tcp_head.seq = response_ack;
        }

        self
    }
}