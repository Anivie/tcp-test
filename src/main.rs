#![feature(cstr_count_bytes)]
#![cfg_attr(debug_assertions, allow(warnings))]

use std::ffi::{c_int, CString};
use std::mem::size_of;
use std::os::raw::c_void;

use bytes::BytesMut;
use rand::random;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};

use crate::raw_bindings::raw_bindings::{AF_INET, htons, in_addr, inet_addr, inet_pton, IP_HDRINCL, iphdr, IPPROTO_IP, IPPROTO_TCP, recvfrom, sendto, setsockopt, SOCK_RAW, sockaddr, sockaddr_in, socket, tcphdr};
use crate::tcp::miao_tcp::TCPPacket;

mod raw_bindings;
mod tcp;

const REMOTE_ADDRESS: &str = "127.0.0.1";
const REMOTE_PORT: u16 = 65534;

#[tokio::main]
async fn main() {
    let socket = unsafe {
        let socket = socket(AF_INET as c_int, SOCK_RAW, IPPROTO_TCP as c_int);
        if socket == -1 {
            panic!("Create socket failed, error: {}", socket);
        }

        let one = 1;
        let opt = setsockopt(socket, IPPROTO_IP as c_int, IP_HDRINCL as c_int, &one as *const i32 as *const c_void, 4);
        if opt == -1 {
            panic!("Create socket failed, error: {}", opt);
        } else {
            println!("Create socket success, socket id: {}", socket);
            println!("Create socket success, opt return: {}", opt);
        }
        socket
    };
    let port: u16 = random();

    let receive_coroutine = tokio::spawn(receive_packet(socket, port));

    tokio::spawn(async move {
        let mut reader = BufReader::new(io::stdin());
        loop {
            let input = read_user_input(&mut reader).await.unwrap();
            if input.trim() == "exit" {
                break;
            }

            send_packet(socket, port).await;
        }
    }).await.unwrap();

    receive_coroutine.await.unwrap();
}

async fn receive_packet(socket: c_int, port: u16) {
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

    println!("receive socket: {}", socket);
    loop {
        let receive_size = unsafe {
            println!("Waiting for packet...");
            recvfrom(
                socket,
                buffer.as_ptr() as *mut u8 as *mut c_void,
                buffer.len(),
                0,
                &mut sockaddr_in as *mut sockaddr_in as *mut sockaddr,
                &mut addr_len as *mut u32,
            )
        };

        unsafe {
            let ip_head = *(buffer.as_ptr() as *const iphdr);
            let tcp_head = *(buffer.as_ptr().offset(size_of::<iphdr>() as isize) as *const tcphdr);
            if ip_head.protocol != 6 {
                println!("Received packet is not a TCP packet, thrown.");
                continue;
            }
            let mut string = String::new();
            string.push_str("Received: {\n");
            string.push_str(format!("  received ip head: {}\n", ip_head).as_str());
            string.push_str(format!("  received tcp head: {}\n", tcp_head).as_str());
            string.push_str(format!("  received size: {}\n", receive_size).as_str());
            string.push_str("}\n");
            println!("{}", string);
        }
    }
}


async fn send_packet(socket: c_int, port: u16) {
    let data = CString::new("miao~").unwrap();

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
        let mut packet = TCPPacket::default(format!("{}:{}", REMOTE_ADDRESS, REMOTE_PORT).as_str(), data, port).unwrap();
        packet.syn_packet();
        packet
    };

    unsafe {
        let sent_size = sendto(
            socket,
            packet.new_bytes().as_ptr() as *const c_void,
            size_of::<iphdr>() + size_of::<tcphdr>() + packet.data_length(),
            0,
            &sockaddr_to as *const sockaddr_in as *const sockaddr,
            size_of::<sockaddr>() as u32
        );

        let mut string = String::new();
        string.push_str("Send: {\n");
        string.push_str(format!(" ip head to send: {}\n", packet.ip_head).as_str());
        string.push_str(format!(" tcp head to send: {}\n", packet.tcp_head).as_str());
        string.push_str(format!(" Send packet with size: {}\n", sent_size).as_str());
        string.push_str("}\n");
        println!("{}", string);
    }
}

async fn read_user_input(reader: &mut BufReader<Stdin>) -> io::Result<String> {
    let mut buffer = String::new();
    reader.read_line(&mut buffer).await?;
    Ok(buffer)
}