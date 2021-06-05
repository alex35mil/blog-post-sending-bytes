use std::io;

use tokio::{io::AsyncWriteExt, net::TcpStream, time};

// `u32` (32-bit unsigned int) is used for storing a length of a message.
// Size of `u32`: 4 bytes.
// https://doc.rust-lang.org/std/mem/fn.size_of.html
const MESSAGE_LENGTH_BUFFER_SIZE: usize = 4;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut socket = TcpStream::connect("127.0.0.1:7080").await?;

    // Waiting for the socket to be writable
    socket.writable().await?;
    println!("socket is writable");

    send_message("hello", &mut socket).await?;
    sleep(3).await;
    send_message("world", &mut socket).await?;

    socket.shutdown().await?;
    println!("all sent. bye.");

    Ok(())
}

async fn send_message(message: &str, socket: &mut TcpStream) -> io::Result<()> {
    println!("[message: {}] starting", message);

    let message_bytes = message.as_bytes();

    let message_len = message_bytes.len() as u32; // cast to u32 is critical to get an array of exactly 4 bytes

    // Getting the memory representation of the message length (u32) as a byte array in big-endian (network) byte order
    let message_len_bytes = message_len.to_be_bytes();

    // Concatenating the message length and the message itself
    let mut bytes = message_len_bytes.to_vec();
    bytes.extend(message_bytes);

    // Simulating a network partition here since we won't get one due to small message size
    //
    // NB: You don't need to do this in the real world app.
    // This is just for illustration of how network can partition data.
    // In general, you'd do:
    // socket.write_all(&bytes).await?;
    let (head, tail) = bytes.split_at(MESSAGE_LENGTH_BUFFER_SIZE + 2);

    socket.write(&head).await?;
    println!("[message: {}] head was written to the socket", message);

    sleep(1).await;

    socket.write(&tail).await?;
    println!("[message: {}] tail was written to the socket", message);

    println!("[message: {}] sent", message);

    Ok(())
}

async fn sleep(secs: u64) {
    time::sleep(std::time::Duration::from_secs(secs)).await
}
