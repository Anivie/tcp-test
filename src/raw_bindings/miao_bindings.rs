#![allow(unused_qualifications)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::mem::size_of;

use rand::random;

use crate::raw_bindings::raw_bindings::{htons, in_addr, inet_addr, inet_ntoa, iphdr, IPPROTO_TCP, ntohs, sockaddr_in, tcphdr};

impl iphdr {
    #[inline]
    pub fn default(data_len: usize, source_addr: &str, destination_addr: &str) -> Self {
        let source_addr = CString::new(source_addr).unwrap();
        let destination_addr = CString::new(destination_addr).unwrap();

        let mut iphdr = unsafe {
            iphdr {
                tos: 0,
                tot_len: (size_of::<iphdr>() + size_of::<tcphdr>() + data_len) as u16,
                id: htons(random()),
                frag_off: 0,
                ttl: 64,
                protocol: IPPROTO_TCP as u8,
                check: 0,
                saddr: inet_addr(source_addr.as_ptr()),
                daddr: inet_addr(destination_addr.as_ptr()),
                ..Default::default()
            }
        };
        iphdr.set_ihl(5);
        iphdr.set_version(4);

        iphdr
    }
}

impl tcphdr {
    #[inline]
    pub fn new() -> Self {
        let mut tcphdr = ::std::mem::MaybeUninit::<Self>::uninit();
        let tcphdr = unsafe {
            std::ptr::write_bytes(tcphdr.as_mut_ptr(), 0, 1);
            tcphdr.assume_init()
        };
        tcphdr
    }

    pub fn default(source_port: u16, destination_port: u16) -> Self {
        let mut tcphdr = Self::new();

        unsafe {
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.source = source_port.to_be();
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.dest = destination_port.to_be();
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.seq = random::<u32>().to_be();
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.ack_seq = 0_u32.to_be();
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_doff(20);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_fin(0);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_syn(0);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_rst(0);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_psh(0);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_ack(0);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.set_urg(0);
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.window = 5840_u16.to_be();
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.check = 0;
            tcphdr.__bindgen_anon_1.__bindgen_anon_2.urg_ptr = 0;
        };

        tcphdr
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

        unsafe {
            write!(
                f,
                "[check: {}, daddr: {}, frag_off: {}, id: {}, ihl: {}, protocol: {}, saddr: {}, tos: {}, tot_len: {}, ttl: {}, version: {}]",
                self.check,
                daddr,
                self.frag_off,
                ntohs(self.id),
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
}