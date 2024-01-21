use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};

use crate::tcp::packet::data::Controller;

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
                let mut packet = controller.make_packet_with_none().to_fin_packet();
                // *controller.spacil.write() = SpacilProcessor::WaveHandshake;
                let sent_size = controller.send_packet(&mut packet);

                tracing::info!("fin data send: {}, with size: {}", packet, sent_size);
            }

            _ => {
                println!("Unknown command: {}", input);
            }
        }
    }
}