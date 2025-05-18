use std::net::TcpStream;
use std::io::{Read, Write};
use crate::tun::TunInterface;
use nix::unistd::read as nix_read;
use nix::unistd::write as nix_write;

pub fn run_client(addr: &str, tun: &TunInterface) {
    let mut stream = TcpStream::connect(addr).expect("Failed to connect to server");
    println!("Connected to server at {}", addr);

    let mut stream_clone = stream.try_clone().expect("Failed to clone TCP stream");
    let tun_fd = tun.fd();

    // TUN -> TCP
    let tun_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 1500];
        loop {
            match nix_read(tun_fd, &mut buf) {
                Ok(n) => {
                    if let Err(e) = stream_clone.write_all(&buf[..n]) {
                        eprintln!("Failed to write to TCP: {}", e);
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

    // TCP -> TUN
    let mut buf = [0u8; 1500];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                if let Err(e) = nix_write(tun_fd, &buf[..n]) {
                    eprintln!("Failed to write to TUN: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading from TCP: {}", e);
                break;
            }
        }
    }

    let _ = tun_thread.join();
}
