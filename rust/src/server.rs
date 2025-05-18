use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use crate::tun::TunInterface;
use nix::unistd::read as nix_read;
use nix::unistd::write as nix_write;

pub fn run_server(addr: &str, tun: &TunInterface) {
    let listener = TcpListener::bind(addr).expect("Failed to bind server socket");
    println!("Server listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected!");
                handle_connection(stream, tun);
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, tun: &TunInterface) {
    let tun_fd = tun.fd();

    let mut tun_to_tcp = stream.try_clone().expect("Failed to clone TCP stream");

    let tun_fd_clone = tun_fd;

    // TUN -> TCP
    let tun_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 1500];
        loop {
            match nix_read(tun_fd_clone, &mut buf) {
                Ok(n) => {
                    if let Err(e) = tun_to_tcp.write_all(&buf[..n]) {
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
