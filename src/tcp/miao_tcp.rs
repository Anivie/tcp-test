use std::ffi::{c_void, CString};
use std::mem::size_of;

use crate::raw_bindings::raw_bindings::{htonl, iphdr, tcphdr};
use crate::tcp::data::PseudoHeader;
use crate::tcp::util::{ToAddress, ToCstring, ToLength};

pub struct TCPPacket {
    pub ip_head: iphdr,
    pub tcp_head: tcphdr,

    data: CString,
    data_vec: Vec<u8>
}

impl TCPPacket {
    pub fn default<A: ToAddress, T: ToCstring + ToLength>(address: A, data: T, source_port: u16) -> Result<TCPPacket, String> {
        let (port, addr) = address.to_address().ok_or("Invalid address")?;

        Ok(TCPPacket {
            ip_head: iphdr::default(data.to_length(), "127.0.0.1", addr),
            tcp_head: tcphdr::default(source_port, port),
            data: data.to_cstring().map_err(|e| e.to_string())?,
            data_vec: Vec::with_capacity(size_of::<Self>())
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_pointer(&mut self) -> *const c_void {
        self.check();

        self.data_vec.clear();
        unsafe {
            let ip = &self.ip_head as *const iphdr as *const u8;
            for e in 0..size_of::<iphdr>() {
                self.data_vec.push(*ip.offset(e as isize))
            }
        };

        unsafe {
            let tcp = &self.tcp_head as *const tcphdr as *const u8;
            for e in 0..size_of::<tcphdr>() {
                self.data_vec.push(*tcp.offset(e as isize))
            }
        };

        for x in self.data.as_bytes() {
            self.data_vec.push(*x)
        }

        self.data_vec.as_ptr() as *const std::os::raw::c_void
    }

    #[inline]
    pub fn size(&self) -> usize {
        size_of::<iphdr>() + size_of::<tcphdr>() + self.data.to_length()
    }

    pub fn syn_packet(&mut self) {
        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.set_syn(1);
        }
    }

    #[allow(dead_code)]
    pub fn change_data<T: ToCstring + ToLength>(&mut self, data: T) -> Result<(), String> {
        self.data = data.to_cstring().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_tcp_check(&self) -> u16 {
        let pseudo_header = PseudoHeader {
            source_address: self.ip_head.saddr,
            dest_address: self.ip_head.daddr,
            placeholder: 0,
            protocol: self.ip_head.protocol,
            tcp_length: unsafe { htonl((size_of::<tcphdr>() + self.data.to_length()) as u32) } as u16
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
        let ip_check_pointer = self as *const TCPPacket as *const u8;

        Self::checksum(ip_check_pointer, self.ip_head.tot_len as usize)
    }

    #[inline]
    #[allow(unused_unsafe)]
    fn check(&mut self) {
        self.ip_head.check = 0;
        self.ip_head.check = {
            let t = self.get_ip_check();
            println!("iphead check: {}", t);
            t
        };

        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.check = 0;
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.check = self.get_tcp_check();
        }
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