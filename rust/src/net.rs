use std::process::{Command, Stdio};

fn run_cmd(cmd: &str) {
    println!("> {}", cmd);
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to execute command");

    if !status.success() {
        eprintln!("command failed: {}", cmd);
    }
}

pub fn ifconfig_tun(name: &str, ip_cidr: &str, mtu: usize) {
    let cmd = format!("ifconfig {} {} mtu {} up", name, ip_cidr, mtu);
    run_cmd(&cmd);
}

pub fn setup_routing_server() {
    run_cmd("sysctl -w net.ipv4.ip_forward=1");

    run_cmd("iptables -t nat -A POSTROUTING -s 10.8.0.0/16 ! -d 10.8.0.0/16 -m comment --comment 'vpndemo' -j MASQUERADE");
    run_cmd("iptables -A FORWARD -s 10.8.0.0/16 -m state --state RELATED,ESTABLISHED -j ACCEPT");
    run_cmd("iptables -A FORWARD -d 10.8.0.0/16 -j ACCEPT")
}

pub fn setup_routing_client() {
    run_cmd("sysctl -w net.ipv4.ip_forward=1");

    run_cmd("iptables -t nat -A POSTROUTING -o tun0 -j MASQUERADE");
    run_cmd("iptables -I FORWARD 1 -i tun0 -m state --state RELATED,ESTABLISHED -j ACCEPT");
    run_cmd("iptables -I FORWARD 1 -o tun0 -j ACCEPT");

    run_cmd("ip route add 0/1 dev tun0");
    run_cmd("ip route add 128/1 dev tun0");
}

pub fn cleanup_routing_server() {
    run_cmd("iptables -t nat -D POSTROUTING -s 10.8.0.0/16 ! -d 10.8.0.0/16 -m comment --comment 'vpndemo' -j MASQUERADE");
    run_cmd("iptables -D FORWARD -s 10.8.0.0/16 -m state --state RELATED,ESTABLISHED -j ACCEPT");
    run_cmd("iptables -D FORWARD -d 10.8.0.0/16 -j ACCEPT");
}

pub fn cleanup_routing_client(server_host: &str) {
    run_cmd("iptables -t nat -D POSTROUTING -o tun0 -j MASQUERADE");
    run_cmd("iptables -D FORWARD -i tun0 -m state --state RELATED,ESTABLISHED -j ACCEPT");
    run_cmd("iptables -D FORWARD -o tun0 -j ACCEPT");

    let del_route = format!("ip route del {}", server_host);
    run_cmd(&del_route);

    run_cmd("ip route del 0/1");
    run_cmd("ip route del 128/1");
}
