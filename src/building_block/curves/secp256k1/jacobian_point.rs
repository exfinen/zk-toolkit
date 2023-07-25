use crate::building_block::{
  curves::secp256k1::{
    affine_point::AffinePoint,
    secp256k1::Secp256k1,
  },
  field::prime_field_elem::PrimeFieldElem,
  zero::Zero,
};
use std::{
  ops::{Add, Mul},
  rc::Rc,
};

#[derive(Clone)]
pub struct JacobianPoint {
  pub curve: Rc<Secp256k1>,
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub z: PrimeFieldElem,
}

impl JacobianPoint {
  pub fn new(
    curve: &Rc<Secp256k1>,
    x: &PrimeFieldElem,
    y: &PrimeFieldElem,
    z: &PrimeFieldElem,
  ) -> Self {
    JacobianPoint {
      curve: curve.clone(),
      x: x.clone(),
      y: y.clone(),
      z: z.clone(),
    }

  }
}

// write tests
impl Zero<JacobianPoint> for JacobianPoint {
  fn is_zero(&self) -> bool {
    self.x.is_zero() && self.y.is_zero()
  }

  fn zero(&self) -> JacobianPoint {
    let one = self.curve.f.elem(&0u8);
    JacobianPoint::new(
      &self.curve,
      &self.x.zero(),
      &self.y.zero(),
      &one,
    )
  }
}

impl From<AffinePoint> for JacobianPoint {
  fn from(p: AffinePoint) -> Self {
    if p.is_zero() {
      panic!("Cannot convert point at infinity to Jacobian point");
    } else {
      JacobianPoint::new(&p.curve, &p.x, &p.y, &p.x.f.elem(&1u8))
    }
  }
}

// TODO acoid using a copy. this is a copy of the same code in affine points
macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl Mul<$rhs> for $target {
      type Output = JacobianPoint;

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
impl_mul!(PrimeFieldElem, JacobianPoint);
impl_mul!(PrimeFieldElem, &JacobianPoint);
impl_mul!(&PrimeFieldElem, JacobianPoint);
impl_mul!(&PrimeFieldElem, &JacobianPoint);

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
            return self.zero();
          }

          let jp = self.clone();

          // formula described in: http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
          // w/ unnecessary computation removed
          let a = &jp.x.sq();
          let b = &jp.y.sq();
          let c = &b.sq();
          let d = &(((&jp.x + b).sq() - a - c) * 2u8);
          let e = &(a * 4u8);
          let e_sq = &e.sq();
          let x3 = e_sq - (d * 2u8);
          let y3 = e * (d - &x3) - (c * 8u8);
          let z3 = &jp.y * 2u8;

          JacobianPoint::new(&self.curve, &x3, &y3, &z3)

        } else {  // when line through p1 and p2 is non-vertical line
          let jp1: JacobianPoint = self.clone();
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

          JacobianPoint::new(&self.curve, &x3, &y3, &z3)
        }
      }
    }
  }
}
impl_add!(JacobianPoint, JacobianPoint);
impl_add!(JacobianPoint, &JacobianPoint);
impl_add!(&JacobianPoint, JacobianPoint);
impl_add!(&JacobianPoint, &JacobianPoint);

impl PartialEq for JacobianPoint {
  // TODO write tests
  fn eq(&self, rhs: &Self) -> bool {
    (&self.x / &self.z == &rhs.x / &rhs.z)
    && (&self.y / &self.z == &rhs.y / &rhs.y)
  }
}

impl Eq for JacobianPoint {}