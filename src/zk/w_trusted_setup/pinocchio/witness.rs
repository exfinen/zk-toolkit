use crate::{
  building_block::field::prime_field_elem::PrimeFieldElem,
  zk::w_trusted_setup::pinocchio::sparse_vec::SparseVec,
};

pub struct Witness {
  sv: SparseVec,
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
    let mid_sv = self.sv.slice(&self.mid_beg, &self.sv.size);
    mid_sv
  }
}
