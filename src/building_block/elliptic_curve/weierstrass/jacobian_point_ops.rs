use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    affine_point::AffinePoint,
    elliptic_curve_point_ops::{
      EllipticCurveField,
      EllipticCurvePointAdd,
      ElllipticCurvePointInv,
    },
    jacobian_point::JacobianPoint,
    new_affine_point::NewAffinePoint,
  },
  zero::Zero,
};

#[derive(Clone)]
pub struct WeierstrassJacobianPointOps<F> {
  f: F,
}

impl<F> WeierstrassJacobianPointOps<F> where F: Clone {
  pub fn new(f: &F) -> Self {
    Self { f: f.clone() }
  }
}

impl<F> EllipticCurveField<F> for WeierstrassJacobianPointOps<F> {
  fn get_field(&self) -> &F {
      &self.f
  }
}

impl<P, E, F> EllipticCurvePointAdd<P, E> for WeierstrassJacobianPointOps<F>
  where
    P: From<JacobianPoint<E>> + Into<JacobianPoint<E>> + Zero<P> + Clone,
    F: AdditiveIdentity<F>
{
  fn add(&self, p1: &P, p2: &P) -> P {
    let f = &self.get_field();

    if p1.is_zero() && p2.is_zero() {  // zero + zero is zero
      F::get_zero(&f)
    } else if p1.is_zero() {  // adding p2 to zero is p2
      p2.clone()
    } else if p2.is_zero() {  // adding p1 to zero is p1
      p1.clone()
    } else if p1.x == p2.x && p1.y != p2.y {  // if line through p1 and p2 is vertical line
      F::get_zero(&f)
    } else if p1.x == p2.x && p1.y == p2.y {  // if adding the same point
      // special case: if y == 0, the tangent line is vertical
      if p1.y.is_zero() || p2.y.is_zero() {
        return F::get_zero(&f);
      }

      // formula described in: http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
      // w/ unnecessary computation removed
      let jp: JacobianPoint<E> = p1.into();

      let a = &jp.x.sq();
      let b = &jp.y.sq();
      let c = &b.sq();
      let d = &(((jp.x + b).sq() - a - c) * 2u8);
      let e = &(a * 3u8);
      let f = &e.sq();
      let x3 = f - (d * 2u8);
      let y3 = e * (d - &x3) - (c * 8u8);
      let z3 = jp.y * 2u8;

      let jp2 = JacobianPoint {
        x: x3,
        y: y3,
        z: z3,
      };
      jp2.into()

    } else {  // when line through p1 and p2 is non-vertical line
      let jp1: JacobianPoint<E> = p1.into();
      let jp2: JacobianPoint<E> = p2.into();

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

      let jp3 = JacobianPoint {
        x: x3,
        y: y3,
        z: z3,
      };
      jp3.into()
    }
  }
}

impl<P, E, F> ElllipticCurvePointInv<P, E> for WeierstrassJacobianPointOps<F>
  where
    E: Zero<E>,
    P: NewAffinePoint<P, E> + Zero<P> + AffinePoint<P, E> {}
