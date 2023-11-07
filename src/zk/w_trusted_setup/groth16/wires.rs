use crate::building_block::curves::mcl::mcl_fr::MclFr;
use core::ops::Index;

// wires:
// 0, 1, .., l, l+1, .., m
// +---------+  +--------+
//  statement    witness
pub struct Wires {
  sv: SparseVec,
  witness_beg: usize,
}

impl Wires {
  // l is index of the last statement wire
  pub fn new(sv: &SparseVec, l: &usize) -> Self {
    Wires {
      sv: sv.clone(),
      witness_beg: l + 1,
    }
  }

  pub fn statement(&self) -> SparseVec {
    let f = &self.f;
    self.sv.slice(&f.elem(&0u8), &f.elem(&self.witness_beg))
  }

  pub fn witness(&self) -> SparseVec {
    let f = &self.f;
    let from = &f.elem(&self.witness_beg);
    let to = &f.elem(&self.sv.size);
    self.sv.slice(from, to)
  }
}

impl Index<usize> for Wires {
  type Output = MclFr;

  fn index(&self, index: usize) -> &Self::Output {
    &self.sv[&index]
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::curves::mcl::mcl_g1::MclG1;

  #[test]
  fn test_wire_indices() {
    // [1,3,35,9,27,8,35]
    let mut sv = SparseVec::new(f, &f.elem(&7u8));
    sv[&f.elem(&0u8)] = f.elem(&1u8);
    sv[&f.elem(&1u8)] = f.elem(&3u8);
    sv[&f.elem(&2u8)] = f.elem(&35u8); // <-- l
    sv[&f.elem(&3u8)] = f.elem(&9u8);
    sv[&f.elem(&4u8)] = f.elem(&27u8);
    sv[&f.elem(&5u8)] = f.elem(&8u8);
    sv[&f.elem(&6u8)] = f.elem(&35u8);

    let w = Wires::new(f, &sv, &2usize);

    let st = &w.statement();
    assert!(st.size == f.elem(&3u8));
    assert!(st[&f.elem(&0u8)] == f.elem(&1u8));
    assert!(st[&f.elem(&1u8)] == f.elem(&3u8));
    assert!(st[&f.elem(&2u8)] == f.elem(&35u8));

    let wit = &w.witness();
    assert!(wit.size == f.elem(&4u8));
    assert!(wit[&f.elem(&0u8)] == f.elem(&9u8));
    assert!(wit[&f.elem(&1u8)] == f.elem(&27u8));
    assert!(wit[&f.elem(&2u8)] == f.elem(&8u8));
    assert!(wit[&f.elem(&3u8)] == f.elem(&35u8));
  }
}













