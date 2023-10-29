# zk-toolkit
Cryptographic primitive library built from scratch

## Goal
To implement cryptographic primitives in the simplest form without using any optimization

## What's implemented so far
- Pinnocio zk-SNARK
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
- Secp256k1
  - Key generation, ECDSA sign/verify
- Weierstrass curve
  - Affine coordinate operations
  - Jacobian coordinate operations
- Generic HMAC
- SHA256 and SHA512 hashers
- Prime finite field
  - Scalar and vector operations

## What's NOT implemented so far
- Arbitrary-precision unsigned integer
- Random number generator

