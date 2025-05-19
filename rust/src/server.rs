use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use crate::tun::TunInterface;
use crate::tls::{load_certs, load_key};
use nix::unistd::{read as nix_read, write as nix_write};
use std::sync::Arc;
use rustls::{ServerConfig, ServerConnection, StreamOwned};

pub fn run_server(addr: &str, tun: &TunInterface) {
    let certs = load_certs("dummy_certs/cert.pem");
    let key = load_key("dummy_certs/key.pem");

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();
    let config = Arc::new(config);

    let listener = TcpListener::bind(addr).expect("Failed to bind server socket");
    println!("Server listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("Client connected!");

                let conn = ServerConnection::new(config.clone()).unwrap();
                let tls_stream = StreamOwned::new(conn, tcp_stream);

                handle_connection(tls_stream, tun);
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_connection(mut tls_stream: StreamOwned<ServerConnection, TcpStream>, tun: &TunInterface) {
    let tun_fd = tun.fd();

    let mut tls_stream_write = tls_stream.sock.try_clone().expect("Clone TLS stream failed");
    let tun_fd_clone = tun_fd;

    let tun_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 1500];
        loop {
            match nix_read(tun_fd_clone, &mut buf) {
                Ok(n) => {
                    if let Err(e) = tls_stream_write.write_all(&buf[..n]) {
                        eprintln!("Failed to write to TLS stream: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from TUN: {}", e);
                    break;
                }
            }
        }
    });

    // TLS -> TUN
    let mut buf = [0u8; 1500];
    loop {
        match tls_stream.read(&mut buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                if let Err(e) = nix_write(tun_fd, &buf[..n]) {
                    eprintln!("Failed to write to TUN: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading from TLS stream: {}", e);
                break;
            }
        }
    }

    let _ = tun_thread.join();
}

