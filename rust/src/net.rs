use std::process::{Command, Stdio};

pub fn ifconfig_tun(name: &str, ip_cidr: &str, mtu: usize) {
    let cmd = format!("ifconfig {} {} mtu {} up", name, ip_cidr, mtu);
    println!("> {}", cmd);

    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to execute ifconfig");

    if !status.success() {
        eprintln!("ifconfig failed with status: {}", status);
        std::process::exit(1);
    }
}
