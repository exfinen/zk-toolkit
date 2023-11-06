use crate::zk::w_trusted_setup::qap::config::SignalId;
use crate::building_block::field::prime_field_elem::PrimeFieldElem;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Term {
  Num(PrimeFieldElem),
  One,
  Out,
  Sum(Box<Term>, Box<Term>),  // Sum will not contain Out and Sum itself
  TmpVar(SignalId),
  Var(String),
}

impl Term {
  pub fn var(name: &str) -> Term {
    Term::Var(name.to_string())
  }
}

impl std::fmt::Debug for Term {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      match self {
        Term::Num(n) => write!(f, "{:?}", n.e),
        Term::One => write!(f, "1"),
        Term::Out => write!(f, "out"),
        Term::Sum(a, b) => write!(f, "({:?} + {:?})", a, b),
        Term::TmpVar(n) => write!(f, "t{:?}", n),
        Term::Var(s) => write!(f, "{:?}", s),
      }
  }
}
