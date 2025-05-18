use std::process::Command;
mod tun;

fn main() {
    let iface = tun::TunInterface::create("tun-vpn", "10.8.0.2", "255.255.0.0", 1500);
    println!("TUN interface {} ready, fd: {}", iface.name, iface.fd());

    println!("ip link command result:");
    let output = Command::new("ip")
        .arg("link")
        .output()
        .expect("failed to run ip link");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}
