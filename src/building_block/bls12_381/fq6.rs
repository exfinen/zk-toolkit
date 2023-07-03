use crate::building_block::bls12_381::fq2::Fq2;

pub struct Fq6 {
    v2: Fq2,
    v1: Fq2,
    v0: Fq2,
}

impl Fq6 {
    pub fn new(v2: Fq2, v1: Fq2, v0: Fq2) -> Self {
        Fq6 { v2, v1, v0 }
    }
}
