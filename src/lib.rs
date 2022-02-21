pub fn test() {
  println!("I'm tested");
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_test() {
    test();
  }
}
