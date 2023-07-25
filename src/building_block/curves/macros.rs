#[macro_export]

macro_rules! impl_mul {
  ($element: ty, $point: ty) => {
    macro_rules! impl_mul {
      ($rhs: ty, $target: ty) => {
        impl Mul<$rhs> for $target {
          type Output = $point;

          fn mul(self, rhs: $rhs) -> Self::Output {
            let mut n = rhs.clone();
            let mut res = self.zero();
            let mut pt_pow_n = self.clone();
            let one = &self.curve.f.elem(&1u8);

            while !&n.is_zero() {
              if !(&n & one).is_zero() {
                res = &res + &pt_pow_n;
              }
              pt_pow_n = &pt_pow_n + &pt_pow_n;
              n >>= &one.e;
            }
            res
          }
        }
      }
    }
    impl_mul!($element, $point);
    impl_mul!($element, &$point);
    impl_mul!(&$element, $point);
    impl_mul!(&$element, &$point);
  }
}
