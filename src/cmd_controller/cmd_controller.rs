use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};

use crate::tcp::packet::data::{Controller, SpacilProcessor};

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
                let mut packet = controller.make_packet_with_none().to_fin_packet(
                    *controller.last_seq_number.read(),
                    *controller.last_ack_seq_number.read()
                );
                let sent_size = controller.send_packet_spacial(&mut packet, SpacilProcessor::WaveHandshake);
                // let sent_size = controller.send_packet(&mut packet);

                tracing::info!("fin data send: {}, with size: {}", packet, sent_size);
            }

            data => {
                let mut packet = controller.make_packet_with_data(data.trim()).to_data_packet(
                    *controller.last_seq_number.read(),
                    *controller.last_ack_seq_number.read()
                );

                let sent_size = controller.send_packet(&mut packet);
                tracing::info!("input data send: {}, with size: {}", packet, sent_size);
                buffer.clear();
            }
        }
    }
}