// src/client.rs
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

use crate::tun::TunInterface;
use nix::unistd::{read as nix_read, write as nix_write};
use openssl::ssl::{SslConnector, SslMethod, SslStream};

pub fn run_client(addr: &str, tun: &TunInterface) {
    println!("[Client] Initializing TLS connector");
    let mut builder = SslConnector::builder(SslMethod::tls())
        .expect("failed to create Connector builder");
    builder
        .set_ca_file("dummy_certs/cert.pem")
        .expect("failed to load CA");
    let connector = Arc::new(builder.build());

    println!("[Client] Connecting TCP to {}", addr);
    let stream = match TcpStream::connect(addr) {
        Ok(s) => {
            println!("[Client] TCP connected to {}", addr);
            s
        }
        Err(e) => {
            eprintln!("[Client] TCP connect error: {}", e);
            return;
        }
    };

    let host = addr.splitn(2, ':').next().unwrap();
    println!("[Client] Performing TLS handshake with SNI = {}", host);
    let ssl_stream: SslStream<TcpStream> =
        match connector.connect(host, stream) {
            Ok(s) => {
                println!("[Client] TLS handshake succeeded");
                s
            }
            Err(e) => {
                eprintln!("[Client] TLS handshake failed: {}", e);
                return;
            }
        };
    let tls = Arc::new(Mutex::new(ssl_stream));
    let tun_fd = tun.fd();

    // TUN -> TLS
    {
        let tls_writer = tls.clone();
        std::thread::spawn(move || {
            println!("[Client] TUN->TLS thread started");
            let mut buf = [0u8; 1500];
            loop {
                let n = match nix_read(tun_fd, &mut buf) {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("[Client] TUN read error: {}", e);
                        break;
                    }
                };
                println!("[Client] Read {} bytes from TUN, writing to TLS", n);
                let mut guard = tls_writer.lock().unwrap();
                if let Err(e) = guard.write_all(&buf[..n]) {
                    eprintln!("[Client] TLS write error: {}", e);
                    break;
                }
                guard.flush().expect("tls flush failed");

            }
            println!("[Client] TUN->TLS thread exiting");
        });
    }

    // TLS -> TUN
    println!("[Client] Entering TLS->TUN loop");
    let mut buf = [0u8; 1500];
    loop {
        let n = {
            let mut guard = tls.lock().unwrap();
            match guard.read(&mut buf) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[Client] TLS read error: {}", e);
                    break;
                }
            }
        };
        println!("[Client] Read {} bytes from TLS, writing to TUN", n);
        if let Err(e) = nix_write(tun_fd, &buf[..n]) {
            eprintln!("[Client] TUN write error: {}", e);
            break;
        }
    }
    println!("[Client] Connection handler exiting");
}
