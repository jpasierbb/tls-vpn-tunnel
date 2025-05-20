# tls-vpn-tunnel

# Generating certs
openssl req -new -x509 -nodes \
  -config req.conf \
  -keyout key.pem \
  -out cert.pem \
  -days 365


# How to run C?
1. Change server IP adress in the include/vpn.h
2. make clean
3. make
4. Start server: sudo ./output/server
5. Start client: sudo ./output/client