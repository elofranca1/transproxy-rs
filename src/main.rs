//! A transparent proxy using Tokio
//! It listens on port 3128 and forwards all traffic to the destination

#![warn(rust_2018_idioms)]

use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpSocket, TcpStream};

use futures::FutureExt;
use socket2::SockRef; // needed for setting socket options
use std::env;
use std::error::Error;

const LISTEN_ADDR: &str = "0.0.0.0:3128";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listen_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| LISTEN_ADDR.to_string());

    println!("Listening on: {}", listen_addr);

    // set socket options on listener
    // note that this sets reuse_port on the listener socket
    // (at least on non-windows platforms)
    let listener = TcpListener::bind(listen_addr).await?;

    // IP_TRANSPARENT is supported on linux only
    #[cfg(target_os = "linux")]
    {
        let listener_socket_ref = SockRef::from(&listener);
        listener_socket_ref.set_ip_transparent(true)?;
    }

    while let Ok((mut inbound, _peer)) = listener.accept().await {
        // now proxy the connection in a separate task
        tokio::spawn(async move {
            if let Err(e) = handle_connection(&mut inbound).await {
                println!("failed to handle connection; error = {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(mut inbound: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let out_socket = TcpSocket::new_v4()?;
    out_socket.set_nodelay(true)?;
    out_socket.set_reuseaddr(true)?;

    socket2::SockRef::from(&out_socket).set_ip_transparent(true)?;

    out_socket.bind(inbound.peer_addr()?)?;
    let mut out_stream = out_socket.connect(inbound.local_addr()?).await?;

    // todo can split directions for better error handling
    copy_bidirectional(&mut inbound, &mut out_stream)
        .map(|r| {
            if let Err(e) = r {
                println!("Failed to transfer; error={}", e);
            }
        })
        .await;

    Ok(())
}
