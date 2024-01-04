#![feature(cstr_count_bytes)]

use std::ffi::{c_int, CString};
use std::mem::size_of;
use std::os::raw::c_void;

use bytes::BytesMut;
use rand::random;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};

use crate::raw_bindings::raw_bindings::{htons, in_addr, inet_addr, IP_HDRINCL, iphdr, IPPROTO_IP, IPPROTO_TCP, PF_INET, recvfrom, sendto, setsockopt, SOCK_RAW, sockaddr, sockaddr_in, socket, tcphdr};
use crate::tcp::data::DataGram;

mod raw_bindings;
mod tcp;

#[tokio::main]
async fn main() {
    let receive_coroutine = tokio::spawn(receive_packet());

    tokio::spawn(async {
        let mut reader = BufReader::new(io::stdin());
        loop {
            let input = read_user_input(&mut reader).await.unwrap();
            // println!("running");
            if input.trim() == "exit" {
                break;
            }

            send_packet().await;
        }
    }).await.unwrap();

    receive_coroutine.await.unwrap();
}

async fn read_user_input(reader: &mut BufReader<Stdin>) -> io::Result<String> {
    let mut buffer = String::new();
    reader.read_line(&mut buffer).await?;
    Ok(buffer)
}

async fn receive_packet() {
    let socket = unsafe {
        let socket = socket(PF_INET as c_int, SOCK_RAW, IPPROTO_TCP as c_int);
        if socket < 0 {
            panic!("Create socket failed, error: {}", socket);
        }
        socket
    };

    let destination_addr = CString::new("127.0.0.1").unwrap();
    let mut receive_addr = unsafe {
        sockaddr_in {
            sin_family: PF_INET as u16,
            sin_port: htons(65534),
            sin_addr: in_addr {
                s_addr: inet_addr(destination_addr.as_ptr()),
            },
            sin_zero: [0; 8],
        }
    };

    let mut addr_len = size_of::<sockaddr>() as u32;

    let mut buffer = BytesMut::with_capacity(4096);
    buffer.resize(4096, 0);

    loop {
        unsafe {
            let r = recvfrom(
                socket,
                buffer.as_ptr() as *mut u8 as *mut c_void,
                buffer.len(),
                0,
                &mut receive_addr as *mut sockaddr_in as *mut sockaddr,
                &mut addr_len as *mut u32,
            );
            println!("Received packet {}, size: {}", receive_addr, r);
        }

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
            string.push_str("}\n");
            println!("{}", string);
        }
    }
}

async fn send_packet() {
    let socket = unsafe {
        let socket = socket(PF_INET as c_int, SOCK_RAW, IPPROTO_TCP as c_int);
        if socket < 0 {
            panic!("Create socket failed, error: {}", socket);
        }

        setsockopt(socket, IPPROTO_IP as c_int, IP_HDRINCL as c_int, 1 as *const c_void, 4);
        socket
    };


    let data = CString::new("miao~").unwrap();

    let iphdr = iphdr::default(data.count_bytes(), "127.0.0.1", "127.0.0.1");
    let tcphdr = tcphdr::default(random(), 65534, data.count_bytes());

    let sockaddr_in = unsafe {
        sockaddr_in {
            sin_family: PF_INET as u16,
            sin_port: htons(65534),
            sin_addr: in_addr {
                s_addr: iphdr.daddr,
            },
            sin_zero: [0; 8],
        }
    };

    let data_gram = DataGram {
        iphdr,
        tcphdr,
        data: data.clone()
    };

    // println!("ip头大小：{}, tcp头大小：{}", size_of::<iphdr>(), size_of::<tcphdr>());

    unsafe {
        let i = sendto(
            socket,
            &data_gram as *const DataGram as *const c_void,
            size_of::<iphdr>() + size_of::<tcphdr>() + data.count_bytes(),
            0,
            &sockaddr_in as *const sockaddr_in as *const sockaddr,
            size_of::<sockaddr>() as u32
        );
        let mut string = String::new();
        string.push_str("Send: {\n");
        string.push_str(format!(" ip head to send: {}\n", iphdr).as_str());
        string.push_str(format!(" tcp head to send: {}\n", tcphdr).as_str());
        string.push_str(format!(" Send packet with size: {}\n", i).as_str());
        string.push_str("}\n");
        println!("{}", string);
    };
}