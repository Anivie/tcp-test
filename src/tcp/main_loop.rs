use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_void;
use std::sync::Arc;

use bytes::BytesMut;
use colored::Colorize;
use log::trace;
use tokio::sync::watch;
use tracing::info;

use crate::raw_bindings::raw_bindings::{AF_INET, in_addr, inet_addr, iphdr, recvfrom, sockaddr, sockaddr_in, tcphdr};
use crate::REMOTE_PORT;
use crate::tcp::packet::data::{Controller, ReceiveData, SpacilProcessor};
use crate::tcp::util::ChangingOrderSizes;

pub async fn receive_packet(controller: Controller) {
    let mut sockaddr_in = unsafe {
        let addr = CString::new("127.0.0.1").unwrap();

        sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: controller.local_port.to_network(),
            sin_addr: in_addr {
                s_addr: inet_addr(addr.as_ptr()),
            },
            sin_zero: [0; 8],
        }
    };

    let (sender, receiver) = watch::channel(None);
    let controller = Arc::new(controller);

    spawn_listener!(controller, receiver, [
        third_handshake_listener,
        data_listener,
        packet_printer,
        fourth_handshake_listener
    ]);

    tokio::spawn(async move {
        let mut addr_len = size_of::<sockaddr>() as u32;
        loop {
            let buffer = {
                let mut buffer = BytesMut::with_capacity(4096);
                buffer.resize(4096, 0);
                buffer
            };

            let receive_size = unsafe {
                recvfrom(
                    controller.socket,
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
                    trace!("{}", "Received packet is not a TCP packet, thrown.".truecolor(25, 160, 60));
                    continue;
                }

                let source_port = tcp_head.__bindgen_anon_1.__bindgen_anon_2.source.to_host();
                let destination_port = tcp_head.__bindgen_anon_1.__bindgen_anon_2.dest.to_host();
                if source_port == controller.local_port {
                    trace!("{}", format!("Received packet from me({}), thrown.", source_port).truecolor(25, 160, 60));
                    continue;
                }

                if !(source_port == controller.local_port && destination_port == REMOTE_PORT) &&
                    !(source_port == REMOTE_PORT && destination_port == controller.local_port) {
                    trace!(
                        "{}",
                        format!(
                            "Received packet does not match the required ports({} to {}), thrown.",
                            source_port,
                            destination_port
                        ).truecolor(25, 160, 60)
                    );
                    continue;
                }

                (ip_head, tcp_head)
            };

            unsafe {
                *controller.last_ack_number.write() = tcp_head.__bindgen_anon_1.__bindgen_anon_2.ack_seq.to_host();
                *controller.last_seq_number.write() = tcp_head.__bindgen_anon_1.__bindgen_anon_2.seq.to_host();
            }

            sender.send(Some(ReceiveData {
                iphdr: ip_head,
                tcphdr: tcp_head,
                packet_size: receive_size as usize,
                data: unsafe {
                    let data_size = receive_size - 20 - (tcp_head.__bindgen_anon_1.__bindgen_anon_2.doff() * 4)as isize;
                    if data_size != 0 {
                        Some(buffer[(20 + (tcp_head.__bindgen_anon_1.__bindgen_anon_2.doff() * 4)) as usize .. receive_size as usize].to_vec())
                    }else {
                        None
                    }
                },
            })).unwrap();
        }
    }).await.unwrap();
}

pub async fn send_packet(controller: Controller) {
    let mut packet = controller.make_packet::<String>(None).to_first_handshake();
    *controller.spacil.write() = SpacilProcessor::InitHandshake;
    let sent_size = controller.send_packet(&mut packet);

    info!("Send second: {}, with size: {}", packet, sent_size);
}
