use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use nix::unistd::{read as nix_read, write as nix_write};
use openssl::ssl::{SslConnector, SslMethod};
use crate::tun::TunInterface;

pub fn run_client(addr: &str, tun: &TunInterface) -> Result<()> {
    // 1) Konfiguracja TLS
    let mut builder = SslConnector::builder(SslMethod::tls())?;
    builder.set_ca_file("dummy_certs/cert.pem")?;
    let connector = Arc::new(builder.build());

    // 2) Nawiąż TCP
    let tcp = TcpStream::connect(addr)?;
    println!("Connected to {}", addr);

    // 3) Handshake TLS
    let tls_stream = connector.connect("vpn", tcp)?;
    let tls = Arc::new(Mutex::new(tls_stream));
    let tun_fd = tun.fd();

    // 4) TUN -> TLS
    {
        let tls_writer = Arc::clone(&tls);
        std::thread::spawn(move || -> Result<()> {
            let mut buf = [0u8; 1500];
            loop {
                let n = nix_read(tun_fd, &mut buf)?;
                let mut guard = tls_writer.lock().unwrap();
                guard.write_all(&buf[..n])?;
            }
        });
    }

    // 5) TLS -> TUN
    let mut buf = [0u8; 1500];
    loop {
        let n = {
            let mut guard = tls.lock().unwrap();
            guard.read(&mut buf)?
        };
        nix_write(tun_fd, &buf[..n])?;
    }
}
