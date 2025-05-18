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
