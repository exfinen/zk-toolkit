use crate::building_block::{
  ec_point::EcPoint,
  field::{Field, FieldElem},
  to_biguint::ToBigUint,
  ec_additive_group_ops::EcAdditiveGroupOps,
  weierstrass_add_ops::JacobianAddOps,
};
use num_bigint::BigUint;

pub struct EcCyclicAdditiveGroup {
  pub f: Field,    // base prime field
  pub f_n: Field,  // field of order n for convenience
  pub g: EcPoint,  // generator point
  pub n: BigUint,  // order of g
  pub ops: Box<dyn EcAdditiveGroupOps>,
}

impl EcCyclicAdditiveGroup {
  pub fn new(
    f: &Field,
    g: EcPoint,
    n: &impl ToBigUint,
    ops: Box<dyn EcAdditiveGroupOps>,
  ) -> Self {
    let n = n.to_biguint();
    let f_n = Field::new(&n);
    EcCyclicAdditiveGroup { f: f.clone(), f_n, g, n, ops }
  }

  pub fn secp256k1() -> EcCyclicAdditiveGroup {
    // base prime field
    let base_field_order = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    let f = &Field::new(&base_field_order);

    // base point of the cyclic group
    let gx = BigUint::parse_bytes(b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap();
    let gy = BigUint::parse_bytes(b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap();
    let g = EcPoint::new(
      &FieldElem::new(f, &gx),
      &FieldElem::new(f, &gy),
    );

    // order of the base point
    let n = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16).unwrap();

    let ops = Box::new(JacobianAddOps::new(f));

    EcCyclicAdditiveGroup::new(f, g, &n, ops)
  }
}