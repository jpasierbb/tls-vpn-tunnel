use tun::{self, Configuration, create};
use tun::Device as _;

use std::os::unix::io::AsRawFd;

pub struct TunInterface {
    pub dev: tun::platform::Device,
    pub name: String,
}

impl TunInterface {
    pub fn create(name: &str, address: &str, netmask: &str, mtu: usize) -> TunInterface {
        let mut config = Configuration::default();
        config
            .name(name)
            .address(address)
            .netmask(netmask)
            .mtu(mtu as i32)
            .up();

        let dev = create(&config).expect("Failed to create TUN device");
        let name = dev.name().to_string();

        println!("Created TUN device: {}", name);

        TunInterface { dev, name }
    }

    pub fn fd(&self) -> std::os::unix::io::RawFd {
        self.dev.as_raw_fd()
    }
}
