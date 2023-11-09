use mcl_rust::*;
use std::sync::Once;

static INIT: Once = Once::new();

pub struct MclInitializer;

impl MclInitializer {
  pub fn init() {
    INIT.call_once(|| {
      if !init(CurveType::BLS12_381) {
        panic!("Failed to initialize mcl");
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn init() {
    MclInitializer::init();
  }
}
