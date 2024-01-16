use std::mem::size_of;

use crate::raw_bindings::raw_bindings::{iphdr, tcphdr};
use crate::tcp::data::PseudoHeader;
use crate::tcp::util::{ToAddress, ToData, ToLength};

pub struct TCPPacket<T: ToData + ToLength> {
    pub ip_head: iphdr,
    pub tcp_head: tcphdr,
    data: T,
}

impl<T: ToData + ToLength> TCPPacket<T> {
    pub fn default<A: ToAddress>(address: A, packet: T, source_port: u16) -> Result<TCPPacket<T>, String> {
        let (port, addr) = address.to_address().ok_or("Invalid address")?;

        let mut data_gram = TCPPacket {
            ip_head: iphdr::default(packet.to_length(), "127.0.0.1", addr),
            tcp_head: tcphdr::default(source_port, port),
            data: packet,
        };

        data_gram.check();
        Ok(data_gram)
    }

    pub fn new_bytes(&self) -> Vec<u8> {
        let size = size_of::<Self>();

        let mut bytes =Vec::with_capacity(size);
        self.to_bytes(&mut bytes);

        bytes
    }

    pub fn to_bytes(&self, back: &mut Vec<u8>) {
        unsafe {
            let ip = &self.ip_head as *const iphdr as *const u8;
            for e in 0..size_of::<iphdr>() {
                back.push(*ip.offset(e as isize))
            }
        };

        unsafe {
            let tcp = &self.tcp_head as *const tcphdr as *const u8;
            for e in 0..size_of::<iphdr>() {
                back.push(*tcp.offset(e as isize))
            }
        };

        for x in self.data.to_data() {
            back.push(*x)
        }
    }

    pub fn syn_packet(&mut self) {
        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.set_syn(1);
        }
        self.check();
    }

    pub fn data_length(&self) -> usize {
        self.data.to_length()
    }

    #[allow(dead_code)]
    pub fn change_data(&mut self, data: T) {
        self.data = data;
        self.check();
    }

    fn get_tcp_check(&self) -> u16 {
        let pseudo_header = PseudoHeader {
            source_address: self.ip_head.saddr,
            dest_address: self.ip_head.daddr,
            placeholder: 0,
            protocol: self.ip_head.protocol,
            tcp_length: (size_of::<tcphdr>() + self.data.to_length()) as u16
        };

        #[allow(dead_code)]
        struct TCPCheck {
            pseudo_header: PseudoHeader,
            tcp_header: tcphdr,
        }

        let tcp_check = TCPCheck {
            pseudo_header,
            tcp_header: self.tcp_head,
        };

        let tcp_check_pointer = &tcp_check as *const TCPCheck as *const u8;
        let tcp_check_len = size_of::<TCPCheck>() + self.data.to_length();

        Self::checksum(tcp_check_pointer, tcp_check_len)
    }

    fn get_ip_check(&self) -> u16 {
        let ip_check_pointer = self as *const TCPPacket<T> as *const u8;

        Self::checksum(ip_check_pointer, self.ip_head.tot_len as usize)
    }

    #[inline]
    #[allow(unused_unsafe)]
    fn check(&mut self) {
        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.check = 0;
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.check = self.get_tcp_check();
        }
        self.ip_head.check = 0;
        self.ip_head.check = self.get_ip_check();
    }

    #[inline]
    fn checksum(buffer: *const u8, len: usize) -> u16 {
        let mut sum = 0u32;
        let mut i = 0;

        unsafe {
            while i < len {
                // 将字节组合成16位整数
                let word = ((*buffer.add(i) as u32) << 8) | (*buffer.add(i + 1)) as u32;
                sum = sum + word;
                i = i + 2;
            }
        }

        // 将溢出加回到低16位
        while (sum >> 16) != 0 {
            sum = (sum & 0xffff) + (sum >> 16);
        }

        // 取反得到校验和
        return (!sum as u16) & 0xffff;
    }
}