# tls-vpn-tunnel

# Generating certs
openssl req -x509 -nodes -days 365 -newkey rsa:2048 -keyout dummy_certs/key.pem -out dummy_certs/cert.pem -config dummy_certs/openssl.cnf

# How to run C?
1. Change server IP adress in the include/vpn.h
2. make clean
3. make
4. Start server: sudo ./output/server
5. Start client: sudo ./output/client