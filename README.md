# zk-toolkit
Library built from scratch to implement zk-protocols.

## Goal
To build a zk library from scratch keeping the implementation as easily understadable as possible so that it can help people including myself who are studying zk technology understand lower level details of it.

## What's implemented so far
- Finite field
- Weierstrass type elliptic curve operations in affine/jacobian coordinates
- ECDSA public key generation, signing and signature verification on Secp256k1 curve
- SHA256 and SHA512 hashers
- Generic HMAC
- Ed25519 public key generation, signing and signature verification on Curve25519 curve
- Bulletproofs
- Pinnocio (equation parser, R1CS, QAP)
- BLS12-381 (Miller's Algorithm, Weil/Tate Pairing, Sign/Verify)

## What's NOT implemented so far
- Big number
- Random number generator
