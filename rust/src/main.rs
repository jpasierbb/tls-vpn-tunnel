mod tun;
mod net;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::process;

fn main() {
    let iface = tun::TunInterface::create("tun-vpn", "10.8.0.2", "255.255.0.0", 1500);
    println!("TUN interface {} ready, fd: {}", iface.name, iface.fd());

    net::ifconfig_tun(&iface.name, "10.8.0.2/16", 1500);
    net::setup_routing(&iface.name);

    // Ctrl+C and cleanup
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nCaught Ctrl+C, cleaning up...");
        net::cleanup_routing("tun-vpn");
        process::exit(0);
    }).expect("Error setting Ctrl+C handler");

    // Waiting
    println!("Press Ctrl+C to exit.");
    while r.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
