#![allow(unused_qualifications)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::mem::size_of;
use rand::random;
use crate::raw_bindings::raw_bindings::{htonl, htons, in_addr, inet_addr, inet_ntoa, iphdr, IPPROTO_TCP, ntohs, sockaddr_in, tcphdr};
use crate::tcp::data::PseudoHeader;

impl iphdr {
    #[inline]
    pub fn default(data_len: usize, source_addr: &str, destination_addr: &str) -> Self {
        let source_addr = CString::new(source_addr).unwrap();
        let destination_addr = CString::new(destination_addr).unwrap();

        let mut iphdr = unsafe {
            iphdr {
                tos: 0,
                tot_len: (size_of::<iphdr>() + size_of::<tcphdr>() + data_len) as u16,
                id: htonl(random()) as u16,
                frag_off: 0,
                ttl: 255,
                protocol: IPPROTO_TCP as u8,
                check: 0,
                saddr: inet_addr(source_addr.as_ptr()),
                daddr: inet_addr(destination_addr.as_ptr()),
                ..Default::default()
            }
        };
        iphdr.set_ihl(5);
        iphdr.set_version(4);
        unsafe {
            iphdr.check = Self::checksum(&iphdr as *const iphdr as *const u8, size_of::<iphdr>());
        }

        iphdr
    }

    unsafe fn checksum(header: *const u8, len: usize) -> u16 {
        let mut sum = 0u32;
        let mut i = 0;

        while i < len {
            // 将字节组合成16位整数
            let word = ((*header.add(i) as u32) << 8) | (*header.add(i + 1)) as u32;
            sum = sum + word;
            i = i + 2;
        }

        // 将溢出加回到低16位
        while (sum >> 16) != 0 {
            sum = (sum & 0xffff) + (sum >> 16);
        }

        // 取反得到校验和
        return (!sum as u16) & 0xffff;
    }
}

impl Display for iphdr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let c_str = unsafe {
            CStr::from_ptr(inet_ntoa(in_addr {
                s_addr: self.saddr
            }))
        };
        let saddr = c_str.to_str().unwrap();

        let c_str = unsafe {
            CStr::from_ptr(inet_ntoa(in_addr {
                s_addr: self.daddr
            }))
        };
        let daddr = c_str.to_str().unwrap();

        write!(
            f,
            "[check: {}, daddr: {}, frag_off: {}, id: {}, ihl: {}, protocol: {}, saddr: {}, tos: {}, tot_len: {}, ttl: {}, version: {}]",
            self.check,
            daddr,
            self.frag_off,
            self.id,
            self.ihl(),
            self.protocol,
            saddr,
            self.tos,
            self.tot_len,
            self.ttl,
            self.version()
        )
    }
}

impl tcphdr {
    pub fn new() -> Self {
        let mut tcphdr = ::std::mem::MaybeUninit::<Self>::uninit();
        let tcphdr = unsafe {
            std::ptr::write_bytes(tcphdr.as_mut_ptr(), 0, 1);
            tcphdr.assume_init()
        };
        tcphdr
    }

    pub fn default(source_port: u16, destination_port: u16, data_size: usize) -> Self {
        let mut tcphdr = Self::new();

        unsafe {
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.source = htons(source_port);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.dest = htons(destination_port);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.seq = random();
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.ack_seq = 0;
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_doff(5);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_syn(1);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.window = htons(5840);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.check = 0;
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.urg_ptr = 0;
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_ack(0);

            let preheader = PseudoHeader {
                source_address: tcphdr.__bindgen_anon_1.__bindgen_anon_2.source as u32,
                dest_address: tcphdr.__bindgen_anon_1.__bindgen_anon_2.dest as u32,
                placeholder: 0,
                protocol: IPPROTO_TCP as u8,
                tcp_length: (size_of::<tcphdr>() + data_size) as u16,
            };

            tcphdr.__bindgen_anon_1.__bindgen_anon_2.check = Self::tcp_checksum(&tcphdr as *const tcphdr as *const u8, size_of::<tcphdr>(), &preheader);
        };

        tcphdr
    }

    unsafe fn tcp_checksum(header: *const u8, len: usize, pseudo_header: &PseudoHeader) -> u16 {
        let mut sum = 0u32;
        let mut i = 0;

        while i < len {
            let word = ((*header.add(i) as u32) << 8) | (*header.add(i + 1)) as u32;
            sum = sum + word;
            i = i + 2;
        }

        sum += (pseudo_header.source_address >> 16) as u32
            + (pseudo_header.source_address & 0xffff) as u32
            + (pseudo_header.dest_address >> 16) as u32
            + (pseudo_header.dest_address & 0xffff) as u32
            + pseudo_header.protocol as u32
            + pseudo_header.tcp_length as u32;

        while (sum >> 16) != 0 {
            sum = (sum & 0xffff) + (sum >> 16);
        }

        return (!sum as u16) & 0xffff;
    }

}

impl Display for tcphdr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "[check: {}, dest: {}, source: {}, doff: {}, fin: {}, psh: {}, rst: {}, seq: {}, syn: {}, ack: {}, ack_seq: {}]",
                self.__bindgen_anon_1.__bindgen_anon_2.check,
                ntohs(self.__bindgen_anon_1.__bindgen_anon_2.dest),
                ntohs(self.__bindgen_anon_1.__bindgen_anon_2.source),
                self.__bindgen_anon_1.__bindgen_anon_2.doff(),
                self.__bindgen_anon_1.__bindgen_anon_2.fin(),
                self.__bindgen_anon_1.__bindgen_anon_2.psh(),
                self.__bindgen_anon_1.__bindgen_anon_2.rst(),
                self.__bindgen_anon_1.__bindgen_anon_2.seq,
                self.__bindgen_anon_1.__bindgen_anon_2.syn(),
                self.__bindgen_anon_1.__bindgen_anon_2.ack(),
                self.__bindgen_anon_1.__bindgen_anon_2.ack_seq
            )
        }
    }
}

impl Display for sockaddr_in {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let c_str = unsafe {
            CStr::from_ptr(inet_ntoa(in_addr {
                s_addr: self.sin_addr.s_addr
            }))
        };
        let string = c_str.to_str().unwrap();
        unsafe {
            write!(f, "[sin_family: {}, sin_port: {}, sin_addr: {}]", self.sin_family, ntohs(self.sin_port), string)
        }
    }
}