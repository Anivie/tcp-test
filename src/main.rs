#![feature(cstr_count_bytes)]
#![feature(let_chains)]
#![feature(lazy_cell)]
#![cfg_attr(debug_assertions, allow(warnings))]

use std::any::Any;
use std::ffi::{c_int, CString};
use std::os::raw::c_void;
use std::sync::{Arc, LazyLock};

use colored::Colorize;
use dashmap::DashMap;
use parking_lot::lock_api::RwLock;
use rand::random;
use tracing::{info, Level};

use crate::cmd_controller::cmd_controller::commandline_listener;
use crate::raw_bindings::raw_bindings::{AF_INET, in_addr, inet_pton, IP_HDRINCL, IPPROTO_IP, IPPROTO_TCP, setsockopt, SOCK_RAW, sockaddr_in, socket};
use crate::tcp::main_loop::{receive_packet, send_packet};
use crate::tcp::packet::data::Controller;
use crate::tcp::util::ChangingOrderSizes;

mod raw_bindings;
mod tcp;
mod cmd_controller;

const REMOTE_ADDRESS: &str = "127.0.0.1";
const REMOTE_PORT: u16 = 65534;

static GLOBAL_MAP: LazyLock<RwLock<parking_lot::RawRwLock, DashMap<&str, Box<dyn Any + Send + Sync>>>>  = LazyLock::new(|| {
    RwLock::new(DashMap::default())
});

#[tokio::main]
#[cfg(target_os = "linux")]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
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
        info!("Start with port: {}", p.to_string().red());
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
        local_port: port,
        sockaddr_to_remote: sockaddr_to,
        address_to_remote: format!("{}:{}", REMOTE_ADDRESS, REMOTE_PORT),
        last_ack_number: Arc::new(RwLock::new(0)),
        last_seq_number: Arc::new(RwLock::new(0)),
    };

    let receive_coroutine = tokio::spawn(receive_packet(control.clone()));
    let user_input_coroutine = tokio::spawn(commandline_listener(control.clone()));
    send_packet(control).await;

    receive_coroutine.await.unwrap();
    user_input_coroutine.await.unwrap();
}