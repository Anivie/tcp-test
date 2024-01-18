#![feature(cstr_count_bytes)]
#![feature(let_chains)]
#![cfg_attr(debug_assertions, allow(warnings))]

use std::ffi::{c_int, CString};
use std::os::raw::c_void;

use rand::random;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};
use tracing::{info, Level};

use crate::raw_bindings::raw_bindings::{AF_INET, in_addr, inet_pton, IP_HDRINCL, IPPROTO_IP, IPPROTO_TCP, setsockopt, SOCK_RAW, sockaddr_in, socket};
use crate::tcp::data::Controller;
use crate::tcp::main_loop::{receive_packet, send_packet};
use crate::tcp::util::ChangingOrderSizes;

mod raw_bindings;
mod tcp;

const REMOTE_ADDRESS: &str = "127.0.0.1";
const REMOTE_PORT: u16 = 65534;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

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
            info!("Create socket success, socket id: {}", socket);
            info!("Create socket success, opt return: {}", opt);
        }
        socket
    };

    let port: u16 = {
        let p: u16 = random();
        info!("Start with port: {}", p);
        p
    };

    let sockaddr_to = unsafe {
        let mut addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: REMOTE_PORT.to_network(),
            ..Default::default()
        };

        let ip = CString::new(REMOTE_ADDRESS).unwrap();
        let res = inet_pton(AF_INET as c_int, ip.as_ptr(), &mut addr.sin_addr as *mut in_addr as *mut c_void);
        if res != 1 {
            panic!("error on inet_pton: {}", res)
        }
        addr
    };

    let control = Controller {
        socket,
        port,
        sockaddr_to,
    };

    let receive_coroutine = tokio::spawn(receive_packet(control));

    tokio::spawn(async move {
        let mut reader = BufReader::new(io::stdin());
        loop {
            send_packet(control).await;
            read_user_input(&mut reader).await.unwrap();
        }
    }).await.unwrap();

    receive_coroutine.await.unwrap();
}

async fn read_user_input(reader: &mut BufReader<Stdin>) -> io::Result<String> {
    let mut buffer = String::new();
    reader.read_line(&mut buffer).await?;
    Ok(buffer.trim_end().to_string())
}