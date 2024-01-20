use std::mem::size_of;

use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};

use crate::GLOBAL_MAP;
use crate::raw_bindings::raw_bindings::{sendto, sockaddr, sockaddr_in};
use crate::tcp::packet::data::Controller;
use crate::tcp::packet::tcp_packet::TCPPacket;

async fn read_user_input(reader: &mut BufReader<Stdin>, buffer: &mut String) -> io::Result<String> {
    reader.read_line(buffer).await?;
    Ok(buffer.trim_end().to_string())
}

pub async fn commandline_listener(controller: Controller) {
    let mut reader = BufReader::new(io::stdin());
    let mut buffer = String::new();
    loop {
        let input = read_user_input(&mut reader, &mut buffer).await.unwrap();

        match input.as_str() {
            "exit" => {
                let mut packet = TCPPacket::default::<_, String>(&controller.address_to_remote, None, controller.local_port).unwrap();
                let sent_size = unsafe {
                    sendto(
                        controller.socket,
                        packet.fin_packet(),
                        packet.len(),
                        0,
                        &controller.sockaddr_to_remote as *const sockaddr_in as *const sockaddr,
                        size_of::<sockaddr>() as u32,
                    )
                };

                tracing::info!("fin data send: {}, with size: {}", packet, sent_size);
                GLOBAL_MAP.write().insert("enable_fin-shaking", Box::new(true));
                // break;
            }

            _ => {
                println!("Unknown command: {}", input);
            }
        }
    }
}