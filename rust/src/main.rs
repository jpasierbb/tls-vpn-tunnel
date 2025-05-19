mod tun;
mod net;
mod server;
mod client;
mod tls;

use ctrlc;
use std::env;
use std::thread;
use std::time::Duration;

use tun::TunInterface;

const SERVER_TCP_ADDR: &str = "0.0.0.0:5555";
const SERVER_VPN_IP: &str = "10.8.0.1";
const CLIENT_VPN_IP: &str = "10.8.0.2";
const CLIENT_TCP_CONNECT_ADDR: &str = "192.168.56.140:5555";
const VPN_MASK: &str = "255.255.0.0";
const VPN_IFACE: &str = "tun0";

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("client").to_string();

    let (vpn_ip, start_fn): (&str, fn(&str, &TunInterface)) = match mode.as_str() {
        "server" => (SERVER_VPN_IP, server::run_server),
        "client" => (CLIENT_VPN_IP, client::run_client),
        _ => {
            eprintln!("Usage: {} [server|client]", args[0]);
            std::process::exit(1);
        }
    };

    let iface = TunInterface::create(VPN_IFACE, vpn_ip, VPN_MASK, 1500);
    net::ifconfig_tun(&iface.name, &format!("{}/16", vpn_ip), 1500);

    match mode.as_str() {
        "server" => net::setup_routing_server(),
        "client" => net::setup_routing_client(),
        _ => {}
    }

    let _cleanup_iface = iface.name.clone();
    let mode_clone = mode.clone();
    ctrlc::set_handler(move || {
        println!("\nCleaning up...");
        match mode_clone.as_str() {
            "server" => net::cleanup_routing_server(),
            "client" => net::cleanup_routing_client(SERVER_VPN_IP),
            _ => {}
        }
        std::process::exit(0);
    }).unwrap();

    println!("TUN interface {} ready.", iface.name);

    let addr = match mode.as_str() {
        "server" => SERVER_TCP_ADDR,
        "client" => CLIENT_TCP_CONNECT_ADDR,
        _ => unreachable!(),
    };

    start_fn(addr, &iface);

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
