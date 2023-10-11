# zk-toolkit
Cryptographic primitives library built from scratch

## Goal
To implement cryptographic primitives in the simplest form without any optimization

## What's implemented so far
- BLS12-381 
  - Extension fields
  - Miller's Algorithm
  - Weil/Tate Pairing
  - Key generation, sign and verify
- Pinnocio zk-SNARKs
  - Equation parser
  - R1CS
  - QAP
  - ** remaing parts to be implemented using BLS12-381 pairing
- Bulletproofs
- Ed25519
  - Key generation, EdDSA sign/veriry
- Weierstrass curve
  - Affine coordinate operations
  - Jacobian coordinate operations
- Secp256k1
  - Key generation, ECDSA sign/verify
- SHA256 and SHA512 hashers
- Generic HMAC

## What's NOT implemented so far
- Big number
- Random number generator

