use crate::building_block::mcl::{
  mcl_g1::MclG1,
  mcl_g2::MclG2,
};

#[allow(non_snake_case)]
pub struct Proof {
  pub A: MclG1,
  pub B: MclG2,
  pub C: MclG1,
}

