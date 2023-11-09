use mcl_rust::*;
use crate::building_block::mcl::{
  mcl_g1::MclG1,
  mcl_g2::MclG2,
  mcl_gt::MclGT,
};

#[derive(Debug, Clone)]
pub struct Pairing;

impl Pairing {
  pub fn e(&self, p1: &MclG1, p2: &MclG2) -> MclGT {
    let mut v = GT::zero();
    pairing(&mut v, &p1.v, &p2.v);
    MclGT::from(&v)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::mcl::{
    mcl_fr::MclFr,
    mcl_initializer::MclInitializer,
  };  

  fn test(
    pairing: &Pairing,
    pair: &dyn Fn(&Pairing, &MclG1, &MclG2) -> MclGT,
    p1: &MclG1,
    p2: &MclG2,
  ) -> bool {
    let ten = MclFr::from(10);
    let ten_p1s = p1 * &ten;

    // test e(p1 + ten_p1s, p2) = e(p1, p2) e(ten_p1s, p2)
    let lhs = pair(pairing, &(p1 + &ten_p1s), p2);
    let rhs1 = pair(pairing, p1, p2);
    let rhs2 = pair(pairing, &ten_p1s, p2);

    let rhs = rhs1 * rhs2;

    lhs == rhs
  }

  fn test_with_generators(
    pair: &dyn Fn(&Pairing, &MclG1, &MclG2) -> MclGT,
  ) {
    let pairing = &Pairing;
    let p1 = MclG1::g();
    let p2 = MclG2::g();
    let res = test(pairing, pair, &p1, &p2);
    assert!(res);
  }

  fn test_with_random_points(
    pair: &dyn Fn(&Pairing, &MclG1, &MclG2) -> MclGT,
  ) {
    let mut errors = 0;
    let num_tests = 1;

    for _ in 0..num_tests {
      let pairing = &Pairing;
      let p1 = MclG1::get_random_point();
      let p2 = MclG2::get_random_point();
      let res = test(pairing, pair, &p1, &p2);
      if res == false {
        errors += 1;
      }
    }
    assert!(errors == 0);
  }

  fn test_plus_to_mul(pair: &dyn Fn(&Pairing, &MclG1, &MclG2) -> MclGT,
  ) {
    let pairing = &Pairing;
    let one = &MclG2::g();

    let p = &(MclG1::g() + MclG1::g());

    let lhs = {
      let p_plus_p = p + p;
      pair(pairing, &p_plus_p, one)
    };

    let rhs = {
      let a = &pair(pairing, &p, one); 
      a * a
    };
    assert!(lhs == rhs);
  }

  #[test]
  fn test_weil_pairing_with_generators() {
    MclInitializer::init();
    test_with_generators(&Pairing::e);
  }

  #[test]
  fn test_weil_pairing_with_random_points() {
    MclInitializer::init();
    test_with_random_points(&Pairing::e);
  }

  #[test]
  fn test_tate_pairing_with_test_plus_to_mul() {
    MclInitializer::init();
    test_plus_to_mul(&Pairing::e);
  }

  #[test]
  fn test_signature_verification() {
    MclInitializer::init();

    let pairing = &Pairing;
    let g1 = &MclG1::g();
    let sk = &MclFr::from(2);
    let pk = &(g1 * sk);

    let m = &b"hamburg steak".to_vec();
    let hash_m = &MclG2::hash_and_map(m);

    // e(pk, H(m)) = e(g1*sk, H(m)) = e(g1, sk*H(m))
    let lhs = pairing.e(pk, hash_m);
    let rhs = pairing.e(g1, &(hash_m * sk));

    assert!(lhs == rhs);
  }
}

