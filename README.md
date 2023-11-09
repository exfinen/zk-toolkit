# zk-toolkit
Cryptographic primitive library built from scratch

## Goal
To implement cryptographic primitives in the simplest form without using any optimization

## What's implemented so far
- Groth16 zk-SNARK
  - An implementation fully based on zk-toolkit
  - An implementation utilizing BLS12-381 curve of external mcl library
- Pinnochio zk-SNARK (protocol 2)
- Common zk-SNARK components
  - Equation parser
  - R1CS
  - QAP
- BLS12-381 
  - Extension fields
  - Miller's algorithm
  - Weil/Tate pairing
  - Key generation, sign and verify
- Bulletproofs
  - Range proof
  - Inner product argument
- Ed25519
  - EdDSA key generation, sign and veriry
- Secp256k1
  - ECDSA key generation, sign and verify
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
