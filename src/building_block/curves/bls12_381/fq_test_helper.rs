use crate::building_block::curves::bls12_381::{
  fq1::Fq1,
  fq2::Fq2,
  fq6::Fq6,
  params::Params as P,
};
use std::rc::Rc;

pub fn get_fq1_values() -> (Fq1, Fq1, Fq1, Fq1) {
  let f_q = &Rc::new(P::base_prime_field());
  let a1 = Fq1::new(f_q, &3u8).negate();
  let b1 = Fq1::new(f_q, &5u8).negate();
  let c1 = Fq1::new(f_q, &7u8).negate();
  let d1 = Fq1::new(f_q, &9u8).negate();
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
