use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::mem::size_of;

use crate::raw_bindings::raw_bindings::{htons, iphdr, tcphdr};
use crate::tcp::data::PseudoHeader;
use crate::tcp::util::{ToAddress, ToCstring, ToLength};

pub struct TCPPacket {
    ip_head: iphdr,
    tcp_head: tcphdr,

    data: CString,
    data_vec: Vec<u8>
}

impl Display for TCPPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n\tTcpHead: {}\n\tIPHead: {}\n}}",
            self.ip_head,
            self.tcp_head
        )
    }
}

impl TCPPacket {
    pub fn syn_packet(&mut self) {
        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.set_syn(1);
        }
    }
}

impl TCPPacket {
    pub fn default<A: ToAddress, T: ToCstring + ToLength>(destination_address: A, data: T, source_port: u16) -> Result<TCPPacket, String> {
        let (port, addr) = destination_address.to_address().ok_or("Invalid address")?;

        Ok(TCPPacket {
            ip_head: iphdr::default(data.to_length(), "127.0.0.1", addr),
            tcp_head: tcphdr::default(source_port, port),
            data: data.to_cstring().map_err(|e| e.to_string())?,
            data_vec: Vec::with_capacity(size_of::<iphdr>() + size_of::<tcphdr>() + data.to_length())
        })
    }

    pub fn as_ptr(&mut self) -> *const u8 {
        self.check();
        self.get_ptr()
    }

    #[allow(clippy::wrong_self_convention)]
    fn get_ptr(&mut self) -> *const u8 {
        self.data_vec.clear();
        let mut offset = 0;
        unsafe {
            let ip = &self.ip_head as *const iphdr as *const u8;
            std::ptr::copy(ip, self.data_vec.as_mut_ptr().offset(offset), size_of::<iphdr>());
            offset += size_of::<iphdr>() as isize;
        };

        unsafe {
            let tcp = &self.tcp_head as *const tcphdr as *const u8;
            std::ptr::copy(tcp, self.data_vec.as_mut_ptr().offset(offset), size_of::<tcphdr>());
            offset += size_of::<tcphdr>() as isize;
        };

        unsafe {
            std::ptr::copy(self.data.as_ptr() as *const u8, self.data_vec.as_mut_ptr().offset(offset), self.data.count_bytes());
        }

        self.data_vec.as_ptr()
    }

    #[inline]
    pub fn len(&self) -> usize {
        size_of::<iphdr>() + size_of::<tcphdr>() + self.data.to_length()
    }

    #[allow(dead_code)]
    pub fn change_data<T: ToCstring + ToLength>(&mut self, data: T) -> Result<(), String> {
        self.data = data.to_cstring().map_err(|e| e.to_string())?;
        Ok(())
    }

    #[inline]
    #[allow(unused_unsafe)]
    fn check(&mut self) {
        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.check = 0;
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.check = self.get_tcp_check();
        }

        self.ip_head.check = 0;
        self.ip_head.check = {
            let t = self.get_ip_check();
            println!("iphead check: {}", t);
            t
        };
    }

    fn get_tcp_check(&self) -> u16 {
        let pseudo_header = PseudoHeader {
            source_address: self.ip_head.saddr,
            dest_address: self.ip_head.daddr,
            placeholder: 0,
            protocol: self.ip_head.protocol,
            tcp_length: unsafe {
                htons((size_of::<tcphdr>() + self.data.to_length())as u16)
            }
        };

        let vec = unsafe {
            let mut vec: Vec<u8> = Vec::with_capacity(size_of::<PseudoHeader>() + size_of::<tcphdr>() + self.data.to_length());
            let mut offset = 0;

            //第一部分：伪头
            std::ptr::copy(
                &pseudo_header as *const PseudoHeader as *const u8,
                vec.as_mut_ptr().offset(offset),
                size_of::<PseudoHeader>()
            );
            offset += size_of::<PseudoHeader>() as isize;

            //第一部分：TCP头
            std::ptr::copy(
                &self.tcp_head as *const tcphdr as *const u8,
                vec.as_mut_ptr().offset(offset),
                size_of::<tcphdr>()
            );
            offset += size_of::<tcphdr>() as isize;

            //第一部分：数据报
            std::ptr::copy(self.data.as_ptr() as *const u8, vec.as_mut_ptr().offset(offset), self.data.count_bytes());

            vec
        };
        //
        // #[allow(dead_code)]
        // struct TCPCheck {
        //     pseudo_header: PseudoHeader,
        //     tcp_header: tcphdr,
        // }
        //
        // let tcp_check = TCPCheck {
        //     pseudo_header,
        //     tcp_header: self.tcp_head,
        // };
        //
        // let tcp_check_pointer = &tcp_check as *const TCPCheck as *const u8;
        // let tcp_check_len = size_of::<TCPCheck>() + self.data.to_length();

        Self::checksum(vec.as_ptr(), vec.len())
    }

    fn get_ip_check(&mut self) -> u16 {
        Self::checksum(self.get_ptr(), self.len())
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