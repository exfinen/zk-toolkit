use crate::{
  building_block::field::prime_field_elem::PrimeFieldElem,
  zk::w_trusted_setup::pinocchio::sparse_vec::SparseVec,
};

pub struct Witness {
  sv: SparseVec,  // includes witness value for `1`
  mid_beg: PrimeFieldElem,
}

impl Witness {
  pub fn new(sv: &SparseVec, mid_beg: &PrimeFieldElem) -> Self {
    Witness {
      sv: sv.clone(),
      mid_beg: mid_beg.clone(),
    }
  }

  pub fn mid(&self) -> SparseVec {
    self.sv.slice(&self.mid_beg, &self.sv.size)
  }

  pub fn io(&self) -> SparseVec {
    let f = &self.mid_beg.f;
    self.sv.slice(&f.elem(&1u8), &self.mid_beg)
  }
}
