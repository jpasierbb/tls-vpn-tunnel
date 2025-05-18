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

pub fn setup_routing(name: &str) {
    run_cmd("sysctl -w net.ipv4.ip_forward=1");
    run_cmd(&format!("ip route add 0/1 dev {}", name));
    run_cmd(&format!("ip route add 128/1 dev {}", name));
}

pub fn cleanup_routing(name: &str) {
    run_cmd(&format!("ip route del 0/1 dev {}", name));
    run_cmd(&format!("ip route del 128/1 dev {}", name));
}

pub fn setup_iptables_server() {
    run_cmd("sysctl -w net.ipv4.ip_forward=1");
    run_cmd("iptables -t nat -A POSTROUTING -s 10.8.0.0/16 ! -d 10.8.0.0/16 -m comment --comment 'vpndemo' -j MASQUERADE");
    run_cmd("iptables -A FORWARD -s 10.8.0.0/16 -m state --state RELATED,ESTABLISHED -j ACCEPT");
    run_cmd("iptables -A FORWARD -d 10.8.0.0/16 -j ACCEPT");
}

pub fn cleanup_iptables_server() {
    run_cmd("iptables -t nat -D POSTROUTING -s 10.8.0.0/16 ! -d 10.8.0.0/16 -m comment --comment 'vpndemo' -j MASQUERADE");
    run_cmd("iptables -D FORWARD -s 10.8.0.0/16 -m state --state RELATED,ESTABLISHED -j ACCEPT");
    run_cmd("iptables -D FORWARD -d 10.8.0.0/16 -j ACCEPT");
}

pub fn setup_iptables_client(name: &str) {
    run_cmd("sysctl -w net.ipv4.ip_forward=1");
    run_cmd(&format!("iptables -t nat -A POSTROUTING -o {} -j MASQUERADE", name));
    run_cmd(&format!("iptables -I FORWARD 1 -i {} -m state --state RELATED,ESTABLISHED -j ACCEPT", name));
    run_cmd(&format!("iptables -I FORWARD 1 -o {} -j ACCEPT", name));
}

pub fn cleanup_iptables_client(name: &str) {
    run_cmd(&format!("iptables -t nat -D POSTROUTING -o {} -j MASQUERADE", name));
    run_cmd(&format!("iptables -D FORWARD -i {} -m state --state RELATED,ESTABLISHED -j ACCEPT", name));
    run_cmd(&format!("iptables -D FORWARD -o {} -j ACCEPT", name));
}
