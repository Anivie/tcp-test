use std::ffi::{c_int, CString};
use std::mem::size_of;
use std::os::raw::c_void;

use bytes::BytesMut;
use tracing::{info, warn};

use crate::{REMOTE_ADDRESS, REMOTE_PORT};
use crate::raw_bindings::raw_bindings::{AF_INET, htons, in_addr, inet_addr, inet_pton, iphdr, ntohs, recvfrom, sendto, sockaddr, sockaddr_in, tcphdr};
use crate::tcp::tcp_packet::TCPPacket;

pub async fn receive_packet(socket: c_int, port: u16) {
    let mut sockaddr_in = unsafe {
        let addr = CString::new("127.0.0.1").unwrap();

        sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(port),
            sin_addr: in_addr {
                s_addr: inet_addr(addr.as_ptr()),
            },
            sin_zero: [0; 8],
        }
    };

    let mut addr_len = size_of::<sockaddr>() as u32;

    let mut buffer = BytesMut::with_capacity(4096);
    buffer.resize(4096, 0);

    loop {
        let receive_size = unsafe {
            recvfrom(
                socket,
                buffer.as_ptr() as *mut u8 as *mut c_void,
                buffer.len(),
                0,
                &mut sockaddr_in as *mut sockaddr_in as *mut sockaddr,
                &mut addr_len as *mut u32,
            )
        };


        let (ip_head, tcp_head) = unsafe {
            let ip_head = *(buffer.as_ptr() as *const iphdr);
            let tcp_head = *(buffer.as_ptr().offset(size_of::<iphdr>() as isize) as *const tcphdr);
            if ip_head.protocol != 6 {
                warn!("Received packet is not a TCP packet, thrown.");
                continue;
            }

            let recv_port = ntohs(tcp_head.__bindgen_anon_1.__bindgen_anon_2.source);
            if recv_port == port {
                info!("Received packet from me, thrown.");
                continue;
            }else if recv_port != REMOTE_PORT {
                info!("Received packet(from {}) is not listening TCP packet, thrown.", recv_port);
                continue;
            }

            (ip_head, tcp_head)
        };

        let mut string = String::new();
        string.push_str("Received: {\n");
        string.push_str(format!("  received ip head: {}\n", ip_head).as_str());
        string.push_str(format!("  received tcp head: {}\n", tcp_head).as_str());
        string.push_str(format!("  received size: {}\n", receive_size).as_str());
        string.push_str("}\n");
        info!("{}", string);
    }
}

pub async fn send_packet(socket: c_int, port: u16) {
    let data = "miao!";

    let sockaddr_to = unsafe {
        let mut addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(REMOTE_PORT),
            ..Default::default()
        };

        let ip = CString::new(REMOTE_ADDRESS).unwrap();
        let res = inet_pton(AF_INET as c_int, ip.as_ptr(), &mut addr.sin_addr as *mut in_addr as *mut c_void);
        if res != 1 {
            panic!("error on inet_pton: {}", res)
        }
        addr
    };

    let mut packet = {
        let address = format!("{}:{}", REMOTE_ADDRESS, REMOTE_PORT);
        let mut packet = TCPPacket::default(address.as_str(), data, port).unwrap();
        packet.syn_packet();
        packet
    };
    // let mut packet_: Vec<u8> = vec![];
    // BASE64_STANDARD.decode_vec("RQA8AAAAAABABj8nfwAAAX8AAAE4Mf/+QsKqTAAAAACgAhbQH4cAAAIEADAEAgAAAAAAAAAAAAAAAAAA", &mut packet_).unwrap();

    unsafe {
        let sent_size = sendto(
            socket,
            packet.as_ptr() as *const c_void,
            packet.len(),
            0,
            &sockaddr_to as *const sockaddr_in as *const sockaddr,
            size_of::<sockaddr>() as u32
        );

        info!("Send: {}, with size: {}", packet, sent_size);
    }
}
