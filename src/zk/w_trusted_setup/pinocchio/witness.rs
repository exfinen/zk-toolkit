use crate::building_block::field::{
  prime_field_elem::PrimeFieldElem,
  sparse_vec::SparseVec,
};

pub struct Witness {
  sv: SparseVec,  // includes witness value for `1`
  pub mid_beg: PrimeFieldElem,
  pub end: PrimeFieldElem,
}

impl Witness {
  pub fn new(sv: &SparseVec, mid_beg: &PrimeFieldElem) -> Self {
    Witness {
      sv: sv.clone(),
      mid_beg: mid_beg.clone(),
      end: &sv.size - sv.f.elem(&1u8),
    }
  }

  pub fn io(&self) -> SparseVec {
    let f = &self.mid_beg.f;
    self.sv.slice(&f.elem(&0u8), &self.mid_beg)
  }

  pub fn mid(&self) -> SparseVec {
    self.sv.slice(&self.mid_beg, &self.sv.size)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::curves::bls12_381::g1_point::G1Point;

  #[test]
  fn test_witness() {
    let f = &G1Point::curve_group();

    // [1,3,35,9,27,8,35]
    let mut sv = SparseVec::new(f, &f.elem(&7u8));
    sv[&f.elem(&0u8)] = f.elem(&1u8);
    sv[&f.elem(&1u8)] = f.elem(&3u8);
    sv[&f.elem(&2u8)] = f.elem(&35u8);
    sv[&f.elem(&3u8)] = f.elem(&9u8);
    sv[&f.elem(&4u8)] = f.elem(&27u8);
    sv[&f.elem(&5u8)] = f.elem(&8u8);
    sv[&f.elem(&6u8)] = f.elem(&35u8);

    let w = Witness::new(&sv, &f.elem(&3u8));

    let io = &w.io();
    assert!(io.size == f.elem(&3u8));
    assert!(io[&f.elem(&0u8)] == f.elem(&1u8));
    assert!(io[&f.elem(&1u8)] == f.elem(&3u8));
    assert!(io[&f.elem(&2u8)] == f.elem(&35u8));

    let mid = &w.mid();
    assert!(mid.size == f.elem(&4u8));
    assert!(mid[&f.elem(&0u8)] == f.elem(&9u8));
    assert!(mid[&f.elem(&1u8)] == f.elem(&27u8));
    assert!(mid[&f.elem(&2u8)] == f.elem(&8u8));
    assert!(mid[&f.elem(&3u8)] == f.elem(&35u8));
  }
}













