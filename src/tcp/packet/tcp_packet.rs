use std::convert::Infallible;
use std::ffi::{c_void, CString};
use std::fmt::{Display, Formatter};
use std::mem::size_of;

use crate::raw_bindings::raw_bindings::{iphdr, tcphdr};
use crate::tcp::packet::data::PseudoHeader;
use crate::tcp::util::{ChangingOrderSizes, ToAddress};

pub struct TCPPacket {
    pub(crate) ip_head: iphdr,
    pub(crate) tcp_head: tcphdr,

    pub(crate) data: CString,
    pub(crate) data_vec: Vec<u8>
}

impl Display for TCPPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n\tsend ip head: {}\n\tsend tcp head: {}\n}}",
            self.ip_head,
            self.tcp_head
        )
    }
}

const LINE_BREAK: &[u8] = &[10];// \n

impl TCPPacket {
    pub fn default<A, T>(destination_address: A, data: Option<T>, source_port: u16) -> Result<TCPPacket, String>
    where A: ToAddress,
          T: Into<Vec<u8>>,
    {
        let (port, addr) = destination_address.to_address().ok_or("Invalid address")?;

        let data = match data {
            None => { CString::default() }
            Some(data) => {
                let mut data: Vec<u8> = data.try_into().map_err(|e: Infallible| e.to_string())?;
                data.extend_from_slice(LINE_BREAK);
                CString::new(data).map_err(|e| e.to_string())?
            }
        };

        let data_len = data.count_bytes();

        Ok(TCPPacket {
            ip_head: iphdr::default(data_len, "127.0.0.1", addr),
            tcp_head: tcphdr::default(source_port, port),
            data,
            data_vec: Vec::with_capacity(size_of::<iphdr>() + size_of::<tcphdr>() + data_len)
        })
    }

    #[inline]
    pub fn as_ptr(&mut self) -> *const c_void {
        self.tcp_check();
        self.calculate_data();
        self.ip_check();
        self.calculate_data();
        self.data_vec.as_ptr() as *const c_void
    }

    fn calculate_data(&mut self) {
        let mut offset = 0;
        self.data_vec.resize(self.len(), 0);

        unsafe {
            let ip = &self.ip_head as *const iphdr as *const u8;
            std::ptr::copy(ip, self.data_vec.as_mut_ptr().offset(offset), size_of::<iphdr>());
            offset += size_of::<iphdr>() as isize;
        }

        unsafe {
            let tcp = &self.tcp_head as *const tcphdr as *const u8;
            std::ptr::copy(tcp, self.data_vec.as_mut_ptr().offset(offset), size_of::<tcphdr>());
            offset += size_of::<tcphdr>() as isize;
        }

        unsafe {
            std::ptr::copy(self.data.as_ptr() as *const u8, self.data_vec.as_mut_ptr().offset(offset), self.data.count_bytes());
        }
    }

    #[inline]
    #[allow(unused_unsafe)]
    pub fn tcp_check(&mut self) {
        unsafe {
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.check = 0;
            self.tcp_head.__bindgen_anon_1.__bindgen_anon_2.check = self.get_tcp_check();
        }
    }

    #[inline]
    pub fn ip_check(&mut self) {
        self.ip_head.check = 0;
        self.ip_head.check = Self::checksum(self.data_vec.as_ptr(), self.len());
    }

    #[inline]
    pub fn len(&self) -> usize {
        size_of::<iphdr>() + size_of::<tcphdr>() + self.data.count_bytes()
    }

    #[allow(dead_code)]
    pub fn change_data<T: Into<Vec<u8>>>(&mut self, data: T) -> Result<(), String> {
        self.data = CString::new(data).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_tcp_check(&mut self) -> u16 {
        let pseudo_header = PseudoHeader {
            source_address: self.ip_head.saddr,
            dest_address: self.ip_head.daddr,
            placeholder: 0,
            protocol: self.ip_head.protocol,
            tcp_length: ((size_of::<tcphdr>() + self.data.count_bytes())as u16).to_network()
        };

        let vec = unsafe {
            let len = size_of::<PseudoHeader>() + size_of::<tcphdr>() + self.data.count_bytes();
            let mut vec: Vec<u8> = Vec::with_capacity(len);
            vec.resize(len, 0);

            let mut offset = 0;

            //第一部分：伪头
            std::ptr::copy(
                &pseudo_header as *const PseudoHeader as *const u8,
                vec.as_mut_ptr().offset(offset),
                size_of::<PseudoHeader>()
            );
            offset += size_of::<PseudoHeader>() as isize;

            //第二部分：TCP头
            std::ptr::copy(
                &self.tcp_head as *const tcphdr as *const u8,
                vec.as_mut_ptr().offset(offset),
                size_of::<tcphdr>()
            );
            offset += size_of::<tcphdr>() as isize;

            //第三部分：数据部
            std::ptr::copy(self.data.as_ptr() as *const u8, vec.as_mut_ptr().offset(offset), self.data.count_bytes());

            vec
        };

        Self::checksum(vec.as_ptr(), vec.len())
    }

    fn checksum(buffer: *const u8, len: usize) -> u16 {
        let mut sum = 0u32;
        let mut i = 0;

        unsafe {
            while i < len {
                // 将字节组合成16位整数
                let word = ((*buffer.add(i + 1) as u32) << 8) | (*buffer.add(i)) as u32;
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