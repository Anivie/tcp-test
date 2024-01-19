use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};

async fn read_user_input(reader: &mut BufReader<Stdin>) -> io::Result<String> {
    let mut buffer = String::new();
    reader.read_line(&mut buffer).await?;
    Ok(buffer.trim_end().to_string())
}