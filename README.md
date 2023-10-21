# zk-toolkit
Cryptographic primitive library built from scratch

## Goal
To implement cryptographic primitives in the simplest form without using any optimization

## What's implemented so far
- Pinnocio verifiable computation
  - Equation parser
  - R1CS
  - QAP
  - Proof Generation/Verification
- BLS12-381 
  - Extension fields
  - Miller's algorithm
  - Weil/Tate pairing
  - Key generation, sign and verify
- Bulletproofs
  - Range proof
  - Inner product argument
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

