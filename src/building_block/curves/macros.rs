#[macro_export]
macro_rules! impl_scalar_mul_point {
  ($multiplier: ty, $multiplicand: ty) => {
    macro_rules! impl_mul {
      ($rhs: ty, $target: ty) => {
        impl Mul<$rhs> for $target {
          type Output = $multiplicand;

          fn mul(self, rhs: $rhs) -> Self::Output {
            let mut n = rhs.clone();
            let mut res = AffinePoint::zero();
            let mut pt_pow_n = self.clone();
            let one = &AffinePoint::curve_group().elem(&1u8);

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
    impl_mul!($multiplier, $multiplicand);
    impl_mul!($multiplier, &$multiplicand);
    impl_mul!(&$multiplier, $multiplicand);
    impl_mul!(&$multiplier, &$multiplicand);
  }
}

#[macro_export]
macro_rules! impl_affine_add {
  ($point: ty) => {
    macro_rules! impl_add {
      ($rhs: ty, $target: ty) => {
        impl Add<$rhs> for $target {
          type Output = $point;

          fn add(self, rhs: $rhs) -> Self::Output {
            match (&self, &rhs) {
              (AffinePoint::AtInfinity, AffinePoint::AtInfinity) => {  // inf + inf is inf
                AffinePoint::AtInfinity
              },
              (AffinePoint::AtInfinity, AffinePoint::Rational { x: _x, y: _y }) => {  // adding inf to rhs is rhs
                rhs.clone()
              },
              (AffinePoint::Rational { x: _x, y: _y }, AffinePoint::AtInfinity) => {  // adding self to inf is self
                self.clone()
              },
              (AffinePoint::Rational { x: x1, y: y1 }, AffinePoint::Rational { x: x2, y: y2 })  // vertical line case
                if x1 == x2 && y1 != y2 => {
                AffinePoint::AtInfinity
              },
              (AffinePoint::Rational { x: x1, y: y1 }, AffinePoint::Rational { x: x2, y: y2 })  // adding the same point
                if x1 == x2 && y1 == y2 => {

                // if y == 0, the denominator of doubling formula is zero and the result is point at infinity
                if y1.is_zero() {
                  return AffinePoint::AtInfinity;
                }

                // differentiate y^2 = x^3 + Ax + B w/ implicit differentiation
                // d/dx(y^2) = d/dx(x^3 + Ax + B)
                // 2y dy/dx = 3x^2 + A
                // dy/dx = (3x^2 + A) / 2y
                //
                // dy/dx is the slope m of the tangent line at the point
                // m = (3x^2 + A) / 2y
                let x1_sq = &x1.sq();
                let m1 = x1_sq + x1_sq + x1_sq;  // x1.sq() * 3u8;
                let m2 = y1 + y1;  // y1 * 2u8;
                let m = m1 * &m2.inv();  // m1 / &m2;

                // equation of intersecting line is
                // y = m(x − p1.x) + p1.y (1)
                //
                // substitute y with (1):
                // (m(x − p1.x) + p1.y)^2 = x^3 + Ax + B
                //
                // moving LHS to RHS, we get:
                // 0 = x^3 - m^2 x^2 + ...  (2)
                //
                // with below equation:
                // (x - r)(x - s)(x - t) = x^3 + (r + s + t)x^2 + (ab + ac + bc)x − abc
                //
                // we know that the coefficient of x^2 term is:
                // r + s + t
                //
                // using (2), the coefficient of x^2 term of the intersecting line is:
                // m^2 = r + s + t
                //
                // since p1 and p2 are the same point, replace r and s w/ p1.x
                // to get the x-coordinate of the point where (1) intersects the curve
                // x3 = m^2 − 2*p1.x
                let p3x = m.sq() - (x1 + x1);  // (x1 * 2u8);

                // then get the y-coordinate by substituting x in (1) w/ x3 to get y3
                // y3 = m(x3 − p1.x) + p1.y
                //
                // reflecting y3 across the x-axis results in the addition result y-coordinate
                // result.y = -1 * y3 = m(p1.x - x3) - p1.y
                let p3y_neg = m * (x1 - &p3x) - y1;

                AffinePoint::new(&p3x, &p3y_neg)
              },
              (AffinePoint::Rational { x: x1, y: y1 }, AffinePoint::Rational { x: x2, y: y2 }) => {  // adding non-inf different points
                // slope m of the line that intersects the curve at p1 and p2:
                // p2.y - p1.y = m(p2.x - p1.x)
                // m(p2.x - p1.x) = p2.y - p1.y
                // m = (p2.y - p1.y) / (p2.x - p1.x)
                let m = (y2 - y1) * (x2 - x1).inv();  // (y2 - y1) / (x2 - x1);

                // then the equation of the line is:
                // y = m(x − p1.x) + p1.y  (1)
                //
                // starting from a curve equation of Weierstrass form:
                // y^2 = x^3 + Ax + B
                //
                // substitute y with (1):
                // (m(x − p1.x) + p1.y)^2 = x^3 + Ax + B
                //
                // moving LHS to RHS, we get:
                // 0 = x^3 - m^2 x^2 + ...  (2)
                //
                // with below equation:
                // (x - r)(x - s)(x - t) = x^3 + (r + s + t)x^2 + (ab + ac + bc)x − abc
                //
                // we know that the coefficient of x^2 term is:
                // r + s + t
                //
                // using (2), the coefficient of x^2 term of the intersecting line is:
                // m^2 = r + s + t
                //
                // substitute r and s with the known 2 roots -p1.x and p2.x:
                // m^2 = p1.x + p2. + t
                // t = m^2 - p1.x - p2.x
                //
                // here t is the x coordinate of the p3 we're trying to find:
                // p3.x = m^2 - p1.x - p2.x
                let p3x = m.sq() - x1 - x2;

                // using (1), find the y-coordinate of the 3rd intersecting point and p3x obtained above
                // y = m(x − p1.x) + p1.y
                // p3.y = m(p3.x − p1.x) + p1.y
                let p3y = m * (&p3x - x1) + y1;

                // then (p3.x, -p3.y) is the result of adding p1 and p2
                AffinePoint::new(&p3x, &-p3y)
              },
            }
          }
        }
      }
    }
    impl_add!($point, $point);
    impl_add!($point, &$point);
    impl_add!(&$point, $point);
    impl_add!(&$point, &$point);
  }
}

#[macro_export]
macro_rules! impl_jacobian_add {
  () => {
    macro_rules! impl_add {
      ($rhs: ty, $target: ty) => {
        impl Add<$rhs> for $target {
          type Output = JacobianPoint;

          // TODO use self and rhs directly and get rid of jp*
          fn add(self, rhs: $rhs) -> Self::Output {
            if self.is_zero() && rhs.is_zero() {  // zero + zero is zero
              self.clone()
            } else if self.is_zero() {  // adding p2 to zero is p2
              rhs.clone()
            } else if rhs.is_zero() {  // adding p1 to zero is p1
              self.clone()
            } else if self.x == rhs.x && self.y != rhs.y {  // if line through p1 and p2 is vertical line
              self.zero()
            } else if self.x == rhs.x && self.y == rhs.y {  // if adding the same point
              // special case: if y == 0, the tangent line is vertical
              if self.y.is_zero() || rhs.y.is_zero() {
                return self.zero().into();
              }
              let jp: JacobianPoint = self.clone();  // TODO use self directly

              // formula described in: http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
              // w/ unnecessary computation removed
              let a = &jp.x.sq();
              let b = &jp.y.sq();
              let c = &b.sq();
              let d = &(((&jp.x + b).sq() - a - c) * 2u8);
              let e = &(a * 3u8);
              let e_sq = &e.sq();
              let x3 = e_sq - (d * 2u8);
              let y3 = e * (d - &x3) - (c * 8u8);
              let z3 = &jp.y * 2u8;

              JacobianPoint::new(&self.curve, &x3, &y3, &z3).into()

            } else {  // when line through p1 and p2 is non-vertical line
              let jp1: JacobianPoint = self.clone();  // TODO use self and rhs directly
              let jp2: JacobianPoint = rhs.clone();

              // formula described in: https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-3.html#addition-add-2007-bl
              // w/ unnecessary computation removed
              let h = jp2.x - &jp1.x;
              let i = (&h * 2).sq();
              let j = &h * &i;
              let r = (jp2.y - &jp1.y) * 2u8;
              let v = jp1.x * &i;
              let x3 = (r.sq() - &j) - (&v * 2u8);
              let y3 = r * (v - &x3) - (jp1.y * (j * 2u8));
              let z3 = h * 2u8;

              JacobianPoint::new(&self.curve, &x3, &y3, &z3).into()
            }
          }
        }
      }
    }
    impl_add!(JacobianPoint, JacobianPoint);
    impl_add!(&JacobianPoint, JacobianPoint);
    impl_add!(JacobianPoint, &JacobianPoint);
    impl_add!(&JacobianPoint, &JacobianPoint);
  }
}
