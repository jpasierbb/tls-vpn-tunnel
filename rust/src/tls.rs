use std::fs::File;
use std::io::BufReader;
use rustls::{Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};

pub fn load_certs(path: &str) -> Vec<Certificate> {
    let mut reader = BufReader::new(File::open(path).unwrap());
    certs(&mut reader).unwrap().into_iter().map(Certificate).collect()
}

pub fn load_key(path: &str) -> PrivateKey {
    let mut reader = BufReader::new(File::open(path).unwrap());
    let keys = pkcs8_private_keys(&mut reader).unwrap();
    PrivateKey(keys[0].clone())
}
