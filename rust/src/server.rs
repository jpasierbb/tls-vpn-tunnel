// src/server.rs
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use nix::unistd::{read as nix_read, write as nix_write};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use crate::tun::TunInterface;

pub fn run_server(addr: &str, tun: &TunInterface) -> Result<()> {
    // TLS setup
    let mut builder = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls())?;
    builder.set_private_key_file("dummy_certs/key.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("dummy_certs/cert.pem")?;
    let acceptor = Arc::new(builder.build());

    // bind na addr (10.8.0.1:5555)
    let listener = TcpListener::bind(addr)?;
    println!("Server listening on {}", addr);

    for stream in listener.incoming() {
        let stream = stream?;
        let acceptor = acceptor.clone();
        let tun_fd = tun.fd();

        std::thread::spawn(move || -> Result<()> {
            // handshake
            let tls_stream = acceptor.accept(stream)?;
            let tls = Arc::new(Mutex::new(tls_stream));

            // TUN -> TLS
            {
                let tls_w = tls.clone();
                std::thread::spawn(move || -> Result<()> {
                    let mut buf = [0u8; 1500];
                    loop {
                        let n = nix_read(tun_fd, &mut buf)?;
                        let mut guard = tls_w.lock().unwrap();
                        guard.write_all(&buf[..n])?;
                    }
                });
            }

            // TLS -> TUN
            let mut buf = [0u8; 1500];
            loop {
                let n = {
                    let mut guard = tls.lock().unwrap();
                    guard.read(&mut buf)?
                };
                nix_write(tun_fd, &buf[..n])?;
            }
        });
    }

    Ok(())
}
