# tls-vpn-tunnel

# Generating certs
openssl req -x509 -nodes -days 365 -newkey rsa:2048 -keyout dummy_certs/key.pem -out dummy_certs/cert.pem -config dummy_certs/openssl.cnf