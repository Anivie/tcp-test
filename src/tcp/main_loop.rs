use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_void;
use std::sync::Arc;

use bytes::BytesMut;
use tokio::sync::watch;
use tracing::{info, warn};

use crate::{REMOTE_ADDRESS, REMOTE_PORT};
use crate::raw_bindings::raw_bindings::{AF_INET, in_addr, inet_addr, iphdr, recvfrom, sendto, sockaddr, sockaddr_in, tcphdr};
use crate::tcp::data::{Controller, ReceiveData};
use crate::tcp::tcp_packet::TCPPacket;
use crate::tcp::util::ChangingOrderSizes;

pub async fn receive_packet(controller: Controller) {
    let mut sockaddr_in = unsafe {
        let addr = CString::new("127.0.0.1").unwrap();

        sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: controller.port.to_network(),
            sin_addr: in_addr {
                s_addr: inet_addr(addr.as_ptr()),
            },
            sin_zero: [0; 8],
        }
    };

    let buffer = {
        let mut buffer = BytesMut::with_capacity(4096);
        buffer.resize(4096, 0);
        buffer
    };

    let (sender, receiver) = watch::channel(None);
    let controller = Arc::new(controller);

    let receiver_inner = receiver.clone();
    let controller_inner = controller.clone();
    tokio::spawn(async move {
        controller_inner.third_handshake(receiver_inner).await;
    });

/*    let receiver_inner = receiver.clone();
    let controller_inner = controller.clone();
    tokio::spawn(async move {
        controller_inner.third_handshake(receiver_inner).await;
    });
*/
    tokio::spawn(async move {
        let mut addr_len = size_of::<sockaddr>() as u32;
        loop {
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
                    warn!("Received packet is not a TCP packet, thrown.");
                    continue;
                }

                let recv_port = tcp_head.__bindgen_anon_1.__bindgen_anon_2.source.to_host();
                if recv_port == controller.port {
                    info!("Received packet from me, thrown.");
                    continue;
                } else if recv_port != REMOTE_PORT {
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

            sender.send(Some(ReceiveData {
                iphdr: ip_head,
                tcphdr: tcp_head,
                data: String::new(),
            })).unwrap();
        }
    }).await.unwrap();
}

pub async fn send_packet(controller: Controller) {
    let address = format!("{}:{}", REMOTE_ADDRESS, REMOTE_PORT);
    let mut packet = TCPPacket::default::<_, String>(address, None, controller.port).unwrap();

    unsafe {
        let sent_size = sendto(
            controller.socket,
            packet.first_handshake(),
            packet.len(),
            0,
            &controller.sockaddr_to as *const sockaddr_in as *const sockaddr,
            size_of::<sockaddr>() as u32
        );

        info!("Send: {}, with size: {}", packet, sent_size);
    }
}
