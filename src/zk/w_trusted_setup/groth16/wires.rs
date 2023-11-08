use crate::building_block::curves::mcl::{
  mcl_fr::MclFr,
  mcl_sparse_vec::MclSparseVec,
};
use core::ops::Index;
use num_traits::Zero;

// wires:
// 0, 1, .., l, l+1, .., m
// +---------+  +--------+
//  statement    witness
pub struct Wires {
  sv: MclSparseVec,
  witness_beg: MclFr,
}

impl Wires {
  // l is index of the last statement wire
  pub fn new(sv: &MclSparseVec, l: &MclFr) -> Self {
    Wires {
      sv: sv.clone(),
      witness_beg: l + MclFr::from(1),
    }
  }

  pub fn statement(&self) -> MclSparseVec {
    self.sv.slice(&MclFr::zero(), &self.witness_beg)
  }

  pub fn witness(&self) -> MclSparseVec {
    let from = &self.witness_beg;
    let to = &self.sv.size;
    self.sv.slice(from, to)
  }
}

impl Index<usize> for Wires {
  type Output = MclFr;

  fn index(&self, index: usize) -> &Self::Output {
    &self.sv[&MclFr::from(index)]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_wire_indices() {
    // [1,3,35,9,27,8,35]
    let mut sv = MclSparseVec::new(&MclFr::from(7));
    sv[&MclFr::from(0)] = MclFr::from(1);
    sv[&MclFr::from(1)] = MclFr::from(3);
    sv[&MclFr::from(2)] = MclFr::from(35);  // <-- l
    sv[&MclFr::from(3)] = MclFr::from(9);
    sv[&MclFr::from(4)] = MclFr::from(27);
    sv[&MclFr::from(5)] = MclFr::from(8);
    sv[&MclFr::from(6)] = MclFr::from(35);

    let w = Wires::new(&sv, &MclFr::from(2));

    let st = &w.statement();
    assert!(st.size == MclFr::from(3));
    assert!(st[&MclFr::from(0)] == MclFr::from(1));
    assert!(st[&MclFr::from(1)] == MclFr::from(3));
    assert!(st[&MclFr::from(2)] == MclFr::from(35));

    let wit = &w.witness();
    assert!(wit.size == MclFr::from(4));
    assert!(wit[&MclFr::from(0)] == MclFr::from(9));
    assert!(wit[&MclFr::from(1)] == MclFr::from(27));
    assert!(wit[&MclFr::from(2)] == MclFr::from(8));
    assert!(wit[&MclFr::from(3)] == MclFr::from(35));
  }
}


