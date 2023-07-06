use crate::building_block::bls12_381::{
  fq1::Fq1,
  fq2::Fq2,
  g1_point::G1Point,
  g2_point::G2Point,
};
use crate::building_block::field::{Field};
use num_bigint::BigUint;
use once_cell::sync::Lazy;

pub static BASE_FIELD: Lazy<Field> = Lazy::new(|| {
  let order = BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).unwrap();
  Field::new(&order)
});

pub static G1_GENERATOR: Lazy<G1Point> = Lazy::new(|| {
  let x = Fq1::new(
    &BASE_FIELD,
    &BigUint::parse_bytes(b"17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb", 16).unwrap(),
  );
  let y = Fq1::new(
    &BASE_FIELD,
    &BigUint::parse_bytes(b"08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1", 16).unwrap(),
  );
  G1Point { x, y }
});

pub static G2_GENERATOR: Lazy<G2Point> = Lazy::new(|| {
  let x1 = BigUint::parse_bytes(b"13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e", 16).unwrap();
  let x2 = BigUint::parse_bytes(b"024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8", 16).unwrap();
  let x = Fq2::new(
    Fq1::new(&BASE_FIELD, &x1),
    Fq1::new(&BASE_FIELD, &x2),
  );
  let y1 = BigUint::parse_bytes(b"0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be", 16).unwrap();
  let y2 = BigUint::parse_bytes(b"0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801", 16).unwrap();
  let y = Fq2::new(
    Fq1::new(&BASE_FIELD, &y1),
    Fq1::new(&BASE_FIELD, &y2),
  );
  G2Point { x, y }
});