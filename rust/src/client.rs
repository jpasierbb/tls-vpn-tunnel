use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, BorrowedFd};
use std::sync::Arc;
use std::io::{Read, Write};

use crate::tun::TunInterface;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::poll::{poll, PollFd, PollFlags};
use nix::unistd::{read as nix_read, write as nix_write};
use openssl::ssl::{SslConnector, SslMethod};

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
        Ok(s) => { println!("[Client] TCP connected to {}", addr); s }
        Err(e) => { eprintln!("[Client] TCP connect error: {}", e); return; }
    };

    let host = addr.splitn(2, ':').next().unwrap();
    println!("[Client] Performing TLS handshake with SNI = {}", host);
    let mut ssl_stream = match connector.connect(host, stream) {
        Ok(s) => { println!("[Client] TLS handshake succeeded"); s }
        Err(e) => { eprintln!("[Client] TLS handshake failed: {}", e); return; }
    };

    // non-blocking
    let tun_fd = tun.fd();
    fcntl(tun_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))
        .expect("tun non-blocking failed");
    ssl_stream.get_ref()
        .set_nonblocking(true)
        .expect("tcp non-blocking failed");

    let mut buf = [0u8; 1500];
    println!("[Client] Entering poll loop");

    loop {
        // Użycie BorrowedFd z unsafe borrow_raw
        let bf_tun = unsafe { BorrowedFd::borrow_raw(tun_fd) };
        let bf_tcp = unsafe { BorrowedFd::borrow_raw(ssl_stream.get_ref().as_raw_fd()) };

        let mut fds = [
            PollFd::new(&bf_tun, PollFlags::POLLIN),
            PollFd::new(&bf_tcp, PollFlags::POLLIN),
        ];
        poll(&mut fds, -1).expect("poll failed");

        // TUN -> TLS
        if let Some(re) = fds[0].revents() {
            if re.contains(PollFlags::POLLIN) {
                match nix_read(tun_fd, &mut buf) {
                    Ok(n) if n > 0 => {
                        println!("[Client] Read {} bytes from TUN, SSL_write...", n);
                        if let Err(e) = ssl_stream.write_all(&buf[..n]) {
                            eprintln!("[Client] SSL write error: {}", e);
                            break;
                        }
                        let _ = ssl_stream.flush();
                    }
                    Ok(_) => {}
                    Err(e) if e == nix::errno::Errno::EAGAIN => {}
                    Err(e) => {
                        eprintln!("[Client] TUN read error: {}", e);
                        break;
                    }
                }
            }
        }

        // TLS -> TUN
        if let Some(re) = fds[1].revents() {
            if re.contains(PollFlags::POLLIN) {
                match ssl_stream.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        println!("[Client] Read {} bytes from SSL, write to TUN...", n);
                        if let Err(e) = nix_write(tun_fd, &buf[..n]) {
                            eprintln!("[Client] TUN write error: {}", e);
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        eprintln!("[Client] SSL read error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    println!("[Client] Exiting");
}
