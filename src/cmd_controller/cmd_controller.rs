use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};

use crate::tcp::packet::data::{Controller, SpacilProcessor};

// This function reads user input from the command line
// It takes a mutable reference to a BufReader and a mutable reference to a String as parameters
// It returns a Result with a String on success and an io::Error on failure
async fn read_user_input(reader: &mut BufReader<Stdin>, buffer: &mut String) -> io::Result<String> {
    reader.read_line(buffer).await?;
    Ok(buffer.trim_end().to_string())
}

// This function listens for commands from the command line
// It takes a Controller as a parameter
// It does not return a value
pub async fn commandline_listener(controller: Controller) {
    // Creating a new BufReader for stdin
    let mut reader = BufReader::new(io::stdin());
    // Creating a new String to hold the user input
    let mut buffer = String::new();
    // Looping indefinitely to continuously read user input
    loop {
        // Reading user input
        let input = read_user_input(&mut reader, &mut buffer).await.unwrap();

        // Matching the user input to perform different actions
        match input.as_str() {
            // If the user input is "exit", send a fin packet
            "exit" => {
                let mut packet = controller.make_packet_with_none().to_fin_packet(
                    *controller.last_seq_number.read(),
                    *controller.last_ack_seq_number.read()
                );
                let sent_size = controller.send_packet_spacial(&mut packet, SpacilProcessor::WaveHandshake);

                tracing::info!("fin data send: {}, with size: {}", packet, sent_size);
            }

            // If the user input is "close", send a data packet with "close" as the data
            "close" => {
                let mut packet = controller.make_packet_with_data("close").to_data_packet(
                    *controller.last_seq_number.read(),
                    *controller.last_ack_seq_number.read()
                );

                let sent_size = controller.send_packet_spacial(&mut packet, SpacilProcessor::WaveHandshake);
                tracing::info!("input data send: {}, with size: {}", packet, sent_size);
                buffer.clear();
            }

            // If the user input is anything else, send a data packet with the user input as the data
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