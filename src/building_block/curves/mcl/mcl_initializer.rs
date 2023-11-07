use mcl_rust::*;

pub struct MclInitializer;

impl MclInitializer {
  pub fn init() {
    if !init(CurveType::BLS12_381) {
      panic!("Failed to initialize mcl");
    }
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
