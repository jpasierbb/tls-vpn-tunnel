# tls-vpn-tunnel

# Generating certs
openssl req -new -x509 -nodes \
  -config openssl.conf \
  -keyout key.pem \
  -out cert.pem \
  -days 365

# How to run C?
1. Change server IP adress in the include/vpn.h
2. make clean
3. make
4. Start server: sudo ./output/server
5. Start client: sudo ./output/client

# How to run Rust?
0. Install rust (docs) and dependencies: sudo apt install libssl-dev pkg-config
1. Change server IP adress in the src/main.rs
2. Go to rust folder and use commands: cargo clean && cargo build
3. Start server: sudo ./target/debug/rust server
4. Start client: sudo ./target/debug/rust client