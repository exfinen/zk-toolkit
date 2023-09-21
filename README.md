# zk-toolkit
Library built from scratch to implement zk-protocols.

## Goal
To build a zk library from scratch keeping the implementation as easily understadable as possible so that it can help people including myself who are studying zk technology understand lower level details of it.

## What's implemented so far
- Basic operations in field calculation (add, sub, mul, inv, pow, sq, negate)
- Weierstrass type elliptic curve add and scalar mul operations in affine/jacobian coordinates
- ECDSA public key generation, signing and signature verification on Secp256k1 curve
- SHA256 and SHA512 hashers
- Generic HMAC
- Ed25519 public key generation, signing and signature verification on Curve25519 curve
- Bulletproofs (inner product argument range proof)
- Pinnocio (equation parser, R1CS, QAP)
- Miller's Algorithm
- Weil Pairing on BLS12-381

## What's NOT implemented so far
- Big number
- Random number generator
