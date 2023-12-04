use std::cmp::Ordering;
use std::convert::TryInto;
use std::io;
use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};
use tokio::task;

// `u32` (32-bit unsigned int) is used for storing a length of a message.
// Size of `u32`: 4 bytes.
// https://doc.rust-lang.org/std/mem/fn.size_of.html
const MESSAGE_LENGTH_BYTES: usize = 4;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7080").await?;

    loop {
        // New connection
        let (socket, addr) = listener.accept().await?;

        // Spawning a new task to handle each connection asynchronously
        task::spawn(handle_connection(socket, addr));
    }
}

async fn handle_connection(socket: TcpStream, addr: SocketAddr) {
    println!("[{}] new connection", addr);

    // Buffer for an incoming message
    let mut buf = Vec::new();
    // Length of an ongoing message
    let mut ongoing_len = None;

    loop {
        // Waiting for the socket to be readable
        socket.readable().await.unwrap();

        match socket.try_read_buf(&mut buf) {
            Ok(0) => {
                // Ok(0) indicates the stream’s read half is closed and will no longer yield data
                println!("[{addr}] nothing left to read, finishing the task and socket.",);
                break;
            }
            Ok(_) => {
                // Some bytes were read and placed in the buffer.
                // First, figuring out the length of the whole message.
                let len = match ongoing_len {
                    None => {
                        // No current length set.
                        // It means that either this is the very first message from this client,
                        // or the previous message was received and `buf` + `len` have been reset.

                        // Taking first 4 bytes out of the buffer IF THEY'VE ALREADY GOT HERE.
                        // This is the length of the whole message.
                        if buf.len() < MESSAGE_LENGTH_BYTES {
                            println!("[{addr}] incomplete length");
                            continue;
                        }
                        let len_bytes = buf
                            .drain(..MESSAGE_LENGTH_BYTES)
                            .collect::<Vec<u8>>()
                            .try_into()
                            .unwrap();

                        // Converting these bytes into u32
                        u32::from_be_bytes(len_bytes)
                    }
                    Some(n) => {
                        // `len` is already set,
                        // which means a head of the message was already received.
                        n
                    }
                };

                match buf.len().cmp(&(len as usize)) {
                    Ordering::Equal | Ordering::Greater => {
                        // Buffer length is equal to or greater than message length,
                        // which means a whole message has been received.
                        let message = buf.drain(..len as usize).collect::<Vec<_>>();
                        let message = std::str::from_utf8(&message).unwrap();
                        println!("[{addr}] message: {message}");
                        // Resetting the ongoing length.
                        ongoing_len = None;
                    }
                    Ordering::Less => {
                        // Buffer length is less then message length,
                        // means the buffer contains only a part of the message
                        ongoing_len = Some(len);
                    }
                }
            }
            // If for whatever reason socket is unreadable, retrying
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => continue,
            Err(err) => panic!("[{}] {}", addr, err),
        }
    }
}
