# zk-toolkit
Cryptographic primitive library built from scratch

## Goal
To implement cryptographic primitives in the simplest form without using any optimization

## What's implemented so far
- BLS12-381 
  - Extension fields
  - Miller's Algorithm
  - Weil/Tate Pairing
  - Key generation, sign and verify
- Pinnocio Verifiable Computation
  - Equation parser
  - R1CS
  - QAP
  - Proof Generation/Verification
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

