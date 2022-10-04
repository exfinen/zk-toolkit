use crate::snarks::config::SignalId;
use crate::building_block::field::FieldElem;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Term {
  Num(FieldElem),
  One,
  Out,
  Sum(Box<Term>, Box<Term>),  // Sum will only not contain Out and Sum itself
  TmpVar(SignalId),
  Var(String),
}

impl std::fmt::Debug for Term {
  fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
      match self {
        Term::Num(n) => print!("{:?}", n.n),
        Term::One => print!("1"),
        Term::Out => print!("out"),
        Term::Sum(a, b) => print!("({:?} + {:?})", a, b),
        Term::TmpVar(n) => print!("t{}", n),
        Term::Var(s) => print!("{}", s),
      };
      Ok(())
  }
}
