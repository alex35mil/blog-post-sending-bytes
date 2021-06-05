use std::{convert::TryInto, io};

use tokio::{net::TcpListener, task};

// `u32` (32-bit unsigned int) is used for storing a length of a message.
// Size of `u32`: 4 bytes.
// https://doc.rust-lang.org/std/mem/fn.size_of.html
const MESSAGE_LENGTH_BUFFER_SIZE: usize = 4;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7080").await?;

    loop {
        // New connection
        let (socket, addr) = listener.accept().await?;

        // Spawning a new task to handle each connection asynchronously
        task::spawn(async move {
            println!("[{}] new connection", addr);

            // Buffer for an incoming message
            let mut buf = Vec::new();
            // Length of the current message
            let mut len = None;

            loop {
                // Waiting for the socket to be readable
                socket.readable().await.unwrap();

                match socket.try_read_buf(&mut buf) {
                    Ok(0) => {
                        // Ok(0) indicates the stream’s read half is closed and will no longer yield data
                        println!("[{}] nothing left to read. we're done here.", addr);
                        break;
                    }
                    Ok(_) => {
                        // Some bytes were read and placed in the buffer.
                        // First, figuring out the length of the whole message.
                        let message_len = match len {
                            None => {
                                // No current length set.
                                // It means that either this is the very first message from this client,
                                // or the previous message was received and `buf` + `len` have been reset.

                                // Taking first 4 bytes out of the buffer.
                                // This is the length of the whole message.
                                let len_bytes = buf
                                    .splice(..MESSAGE_LENGTH_BUFFER_SIZE, vec![])
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

                        if message_len as usize == buf.len() {
                            // Buffer length is equal to message length,
                            // means the whole message has been received
                            let message = std::str::from_utf8(&buf).unwrap();
                            println!("[{}] message: {}", addr, message);
                            // Resetting the buffer and the current length
                            buf.clear();
                            len = None;
                        } else if message_len as usize > buf.len() {
                            // Buffer length is less then message length,
                            // means the buffer contains only a part of the message
                            len = Some(message_len);
                        } else {
                            panic!("Message length < current buffer");
                        }
                    }
                    // If for whatever reason socket is unreadable, retrying
                    Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => continue,
                    Err(err) => panic!("[{}] {}", addr, err),
                }
            }
        });
    }
}
