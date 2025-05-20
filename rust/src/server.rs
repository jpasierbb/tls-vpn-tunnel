// src/server.rs
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

use crate::tun::TunInterface;
use nix::unistd::{read as nix_read, write as nix_write};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslStream};

pub fn run_server(addr: &str, tun: &TunInterface) {
    println!("[Server] Initializing TLS acceptor");
    let mut tls_builder = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls())
        .expect("failed to create Acceptor builder");
    tls_builder
        .set_private_key_file("dummy_certs/key.pem", SslFiletype::PEM)
        .expect("failed to load key");
    tls_builder
        .set_certificate_chain_file("dummy_certs/cert.pem")
        .expect("failed to load cert");
    let acceptor = Arc::new(tls_builder.build());

    println!("[Server] Binding TCP listener on {}", addr);
    let listener = TcpListener::bind(addr).expect("Failed to bind server socket");

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[Server] TCP accept error: {}", e);
                continue;
            }
        };
        let peer = stream.peer_addr().ok();
        println!("[Server] New TCP connection from {:?}", peer);

        let acceptor = acceptor.clone();
        let tun_fd = tun.fd();
        std::thread::spawn(move || {
            println!("[Server:{:?}] Starting TLS handshake", peer);
            let ssl_stream: SslStream<std::net::TcpStream> =
                acceptor.accept(stream).expect("TLS accept failed");
            println!("[Server:{:?}] TLS handshake completed", peer);

            let tls = Arc::new(Mutex::new(ssl_stream));

            // TUN -> TLS
            {
                let tls_writer = tls.clone();
                std::thread::spawn(move || {
                    println!("[Server:{:?}] TUN->TLS thread started", peer);
                    let mut buf = [0u8; 1500];
                    loop {
                        let n = match nix_read(tun_fd, &mut buf) {
                            Ok(n) => n,
                            Err(e) => {
                                eprintln!("[Server:{:?}] TUN read error: {}", peer, e);
                                break;
                            }
                        };
                        println!("[Server:{:?}] Read {} bytes from TUN, writing to TLS", peer, n);
                        let mut guard = tls_writer.lock().unwrap();
                        if let Err(e) = guard.write_all(&buf[..n]) {
                            eprintln!("[Server:{:?}] TLS write error: {}", peer, e);
                            break;
                        }
                        guard.flush().expect("tls flush failed");

                    }
                    println!("[Server:{:?}] TUN->TLS thread exiting", peer);
                });
            }

            // TLS -> TUN
            println!("[Server:{:?}] Entering TLS->TUN loop", peer);
            let mut buf = [0u8; 1500];
            loop {
                let n = {
                    let mut guard = tls.lock().unwrap();
                    match guard.read(&mut buf) {
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("[Server:{:?}] TLS read error: {}", peer, e);
                            break;
                        }
                    }
                };
                println!("[Server:{:?}] Read {} bytes from TLS, writing to TUN", peer, n);
                if let Err(e) = nix_write(tun_fd, &buf[..n]) {
                    eprintln!("[Server:{:?}] TUN write error: {}", peer, e);
                    break;
                }
            }
            println!("[Server:{:?}] Connection handler exiting", peer);
        });
    }
}
