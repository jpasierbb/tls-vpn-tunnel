mod tun;
mod net;
mod server;
mod client;

use anyhow::Result;
use ctrlc;
use std::{env, thread};
use std::time::Duration;
use tun::TunInterface;

const SERVER_TCP_ADDR: &str       = "0.0.0.0:5555";
const SERVER_VPN_IP: &str         = "10.8.0.1";
const CLIENT_VPN_IP: &str         = "10.8.0.2";
const CLIENT_TCP_CONNECT_ADDR: &str = "192.168.10.10:5555";
const VPN_MASK: &str              = "255.255.0.0";
const VPN_IFACE: &str             = "tun0";

fn main() {
    // 1) Parsujemy tryb
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1)
        .map(String::as_str)
        .unwrap_or("client");

    // 2) Wybieramy IP i funkcję startującą, która zwraca Result<()>
    let (vpn_ip, start_fn): (&str, fn(&str, &TunInterface) -> Result<()>) =
        match mode {
            "server" => (SERVER_VPN_IP, server::run_server),
            "client" => (CLIENT_VPN_IP, client::run_client),
            _ => {
                eprintln!("Usage: {} [server|client]", args[0]);
                std::process::exit(1);
            }
        };

    // 3) Tworzymy TUN
    let iface = TunInterface::create(VPN_IFACE, vpn_ip, VPN_MASK, 1500);
    net::ifconfig_tun(&iface.name, &format!("{}/16", vpn_ip), 1500);

    // 4) Konfigurujemy routing / iptables
    match mode {
        "server" => net::setup_routing_server(),
        "client" => net::setup_routing_client(),
        _ => unreachable!(),
    }

    // 5) Ctrl+C cleanup tylko routingu
    let mode_clone = mode.to_string();
    ctrlc::set_handler(move || {
        println!("\nCleaning up routing rules...");
        if mode_clone == "server" {
            net::cleanup_routing_server();
        } else {
            net::cleanup_routing_client(SERVER_VPN_IP);
        }
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    println!("TUN interface {} ready.", iface.name);

    // 6) Połącz TCP/TLS + TUN
    let addr = if mode == "server" {
        SERVER_TCP_ADDR
    } else {
        CLIENT_TCP_CONNECT_ADDR
    };

    // Wywołujemy i jednorazowo odpytujemy o błąd
    if let Err(e) = start_fn(addr, &iface) {
        eprintln!("Error in {} mode: {:#}", mode, e);
        std::process::exit(1);
    }

    // 7) Utrzymujemy program przy życiu
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
