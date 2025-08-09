# tls-vpn-tunnel

## Overview
This repository contains a proof-of-concept **application-layer Virtual Private Network (VPN)** implemented entirely in **C** and **Rust**, featuring a TLS v1.3 encrypted tunnel between a VPN client and server.

## Key Features
- **TLS v1.3 Encryption** — All traffic between client and server is encapsulated in a secure, authenticated TLS tunnel.
- **User-Space VPN** — Operates entirely in user space, without kernel modules or administrative network stack changes beyond interface configuration.
- **TUN Interface Integration** — Uses a virtual TUN device to capture and inject raw IP packets.
- **Certificate-Based Authentication** — Ensures mutual trust between client and server before tunnel establishment.
- **Full Traffic Redirection** — Dynamically modifies routing tables to transparently send all client traffic through the VPN.
- **NAT Handling on Server** — Uses SNAT/DNAT with `iptables` for secure forwarding to external networks.

# Generating certs
Use below commands in the dummy_certs/ folder:  
**openssl req -new -x509 -nodes \
  -config openssl.cnf \
  -keyout key.pem \
  -out cert.pem \
  -days 365**

# How to run C?
1. Change server IP adress in the **include/vpn.h** - there's no need to generate new certs after changing.
2. **make clean**
3. **make**
4. Start server: **sudo ./output/server**
5. Start client: **sudo ./output/client**

# How to run Rust?
0. Install rust (docs) and dependencies: **sudo apt install libssl-dev pkg-config**
1. Change server IP adress in the src/main.rs.
2. IF you changed IP, you need to **modify openssl.cnf** file and generate new certs.
3. Go to rust folder and use commands: **cargo clean && cargo build**
4. Start server: **sudo ./target/debug/rust server**
5. Start client: **sudo ./target/debug/rust client**
