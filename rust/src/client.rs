use std::net::TcpStream;
use std::io::{Read, Write};
use crate::tun::TunInterface;
use nix::unistd::{read as nix_read, write as nix_write};
use std::sync::{Arc, Mutex};
use rustls::{ClientConfig, ClientConnection, OwnedTrustAnchor, RootCertStore, StreamOwned};
use webpki_roots::TLS_SERVER_ROOTS;

pub fn run_client(addr: &str, tun: &TunInterface) {
    let mut root_store = RootCertStore::empty();
    root_store.add_trust_anchors(
        TLS_SERVER_ROOTS.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject, ta.spki, ta.name_constraints
            )
        })
    );

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let config = Arc::new(config);

    let tcp_stream = TcpStream::connect(addr).expect("Failed to connect to server");
    let conn = ClientConnection::new(config.clone(), "localhost".try_into().unwrap()).unwrap();
    let tls_stream = StreamOwned::new(conn, tcp_stream);

    println!("TLS handshake completed");

    let tls_stream = Arc::new(Mutex::new(tls_stream));
    let tun_fd = tun.fd();
    let tls_writer = tls_stream.clone();

    // TUN -> TLS (pisanie)
    let tun_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 1500];
        loop {
            match nix_read(tun_fd, &mut buf) {
                Ok(n) => {
                    let mut stream = tls_writer.lock().unwrap();
                    if let Err(e) = stream.write_all(&buf[..n]) {
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

    // TLS -> TUN (czytanie)
    let mut buf = [0u8; 1500];
    loop {
        let n = {
            let mut stream = tls_stream.lock().unwrap();
            match stream.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Error reading from TLS stream: {}", e);
                    break;
                }
            }
        };

        if let Err(e) = nix_write(tun_fd, &buf[..n]) {
            eprintln!("Failed to write to TUN: {}", e);
            break;
        }
    }

    let _ = tun_thread.join();
}
