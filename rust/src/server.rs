use std::net::TcpListener;
use std::os::unix::io::{AsRawFd, BorrowedFd};
use std::sync::Arc;
use std::io::{Read, Write};

use crate::tun::TunInterface;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::poll::{poll, PollFd, PollFlags};
use nix::unistd::{read as nix_read, write as nix_write};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

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
            Err(e) => { eprintln!("[Server] TCP accept error: {}", e); continue; }
        };
        let peer = stream.peer_addr().ok();
        println!("[Server] New connection from {:?}", peer);

        let acceptor = acceptor.clone();
        let tun_fd = tun.fd();
        std::thread::spawn(move || {
            println!("[Server:{:?}] Performing TLS handshake", peer);
            let mut ssl_stream = acceptor.accept(stream).expect("TLS accept failed");
            println!("[Server:{:?}] Handshake done", peer);

            fcntl(tun_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();
            ssl_stream.get_ref()
                .set_nonblocking(true)
                .unwrap();

            let mut buf = [0u8; 1500];
            println!("[Server:{:?}] Entering poll loop", peer);

            loop {
                let bf_tun = unsafe { BorrowedFd::borrow_raw(tun_fd) };
                let bf_tcp = unsafe { BorrowedFd::borrow_raw(ssl_stream.get_ref().as_raw_fd()) };
                let mut fds = [
                    PollFd::new(&bf_tun, PollFlags::POLLIN),
                    PollFd::new(&bf_tcp, PollFlags::POLLIN),
                ];
                poll(&mut fds, -1).unwrap();

                // TUN -> TLS
                if let Some(re) = fds[0].revents() {
                    if re.contains(PollFlags::POLLIN) {
                        if let Ok(n) = nix_read(tun_fd, &mut buf) {
                            if n > 0 {
                                let _ = ssl_stream.write_all(&buf[..n]);
                                let _ = ssl_stream.flush();
                            }
                        }
                    }
                }

                // TLS -> TUN
                if let Some(re) = fds[1].revents() {
                    if re.contains(PollFlags::POLLIN) {
                        if let Ok(n) = ssl_stream.read(&mut buf) {
                            if n > 0 {
                                let _ = nix_write(tun_fd, &buf[..n]);
                            }
                        }
                    }
                }
            }
        });
    }
}
