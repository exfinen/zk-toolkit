use crate::building_block::{
  field::prime_field::PrimeField,
  elliptic_curve::weierstrass::curves::bls12_381::{
    fq1::{Fq1, FIELD_ORDER},
    fq2::Fq2,
    fq6::Fq6,
  },
};
use once_cell::sync::Lazy;

pub fn get_fq1_values() -> (Fq1, Fq1, Fq1, Fq1) {
  let f = &PrimeField::new(Lazy::get(&FIELD_ORDER).unwrap());
  let a1 = Fq1::new(f, &3u8).negate();
  let b1 = Fq1::new(f, &5u8).negate();
  let c1 = Fq1::new(f, &7u8).negate();
  let d1 = Fq1::new(f, &9u8).negate();
  (a1, b1, c1, d1)
}

pub fn get_fq2_values() -> (Fq2, Fq2, Fq2, Fq2) {
  let (a1, b1, c1, d1) = get_fq1_values();
  let a2 = Fq2::new(&a1, &b1);
  let b2 = Fq2::new(&b1, &c1);
  let c2 = Fq2::new(&c1, &d1);
  let d2 = Fq2::new(&d1, &a1);
  (a2, b2, c2, d2)
}

pub fn get_fq6_values() -> (Fq6, Fq6, Fq6, Fq6) {
  let (a2, b2, c2, d2) = get_fq2_values();
  let a6 = Fq6::new(&a2, &b2, &c2);
  let b6 = Fq6::new(&b2, &c2, &d2);
  let c6 = Fq6::new(&c2, &d2, &a2);
  let d6 = Fq6::new(&d2, &a2, &b2);
  (a6, b6, c6, d6)
}
