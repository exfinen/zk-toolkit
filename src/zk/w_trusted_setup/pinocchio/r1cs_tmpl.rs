use crate::building_block::field::{
  prime_field::PrimeField,
  prime_field_elem::PrimeFieldElem,
};
use crate::zk::w_trusted_setup::pinocchio::{
  term::Term,
  gate::Gate,
  constraint::Constraint,
};
use std::collections::HashMap;

use super::sparse_vec::SparseVec;

pub struct R1CSTmpl<'a> {
  pub f: &'a PrimeField,
  pub constraints: Vec<Constraint>,
  pub witness: Vec<Term>,
  pub indices: HashMap<Term, PrimeFieldElem>,
  pub mid_beg: PrimeFieldElem,
}

impl<'a> R1CSTmpl<'a> {
  pub fn new(f: &'a PrimeField) -> Self {
    let mut tmpl = R1CSTmpl {
      f,
      constraints: vec![],
      witness: vec![],
      indices: HashMap::<Term, PrimeFieldElem>::new(),
      mid_beg: f.elem(&0u8),
    };
    // add `1` at index 0
    tmpl.witness.push(Term::One);
    tmpl.indices.insert(Term::One, f.elem(&0u8));

    tmpl
  }

  pub fn categorize_witness_terms(
    &self,
    t: &Term,
    inputs: &mut Vec<Term>,
    mid: &mut Vec<Term>,
  ) {
    match t {
      Term::One => (),   // not categorized as inputs or mid
      Term::Num(_) => (),  // Num is represented as multiple of Term::One, so not adding to witness
      Term::Out => (),  // not categorized as inputs or mid
      Term::Var(_) => if !inputs.contains(&t) { inputs.push(t.clone()) },
      Term::TmpVar(_) => if !mid.contains(&t) { mid.push(t.clone()) },
      Term::Sum(a, b) => {
        self.categorize_witness_terms(&a, inputs, mid);
        self.categorize_witness_terms(&b, inputs, mid);
      },
    }
  }

  fn build_constraint_vec(&mut self, f: &PrimeField, vec: &mut SparseVec, term: &Term) {
    match term {
      Term::Sum(a, b) => {
        self.build_constraint_vec(f, vec, &a);
        self.build_constraint_vec(f, vec, &b);
      },
      Term::Num(n) => {
        vec.set(&0u8, n);  // Num is represented as Term::One at index 0 times n
      },
      x => {
        let index = self.indices.get(&x).unwrap();
        vec.set(index, &1u8);
      },
    }
  }

  // build witness vector whose elements in the following order:
  // 1, inputs, Out, mid
  fn build_witness(&mut self, f: &PrimeField, inputs: &Vec<Term>, mid: &Vec<Term>) {
    let mut i = f.elem(&1u8);  // `1` has already been added in new function

    for x in inputs {
      self.witness.push(x.clone());
      self.indices.insert(x.clone(), i.clone());
      i.inc();
    }
    self.witness.push(Term::Out);
    self.indices.insert(Term::Out, i.clone());
    i.inc();

    self.mid_beg = i.clone();

    for x in mid {
      self.witness.push(x.clone());
      self.indices.insert(x.clone(), i.clone());
      i.inc();
    }
  }

  pub fn from_gates(f: &'a PrimeField, gates: &[Gate]) -> Self {
    let mut tmpl = R1CSTmpl::new(f);

    // categoraize terms contained in gates to inputs and mid
    let mut inputs = vec![];
    let mut mid = vec![];

    for gate in gates {
      tmpl.categorize_witness_terms(&gate.a, &mut inputs, &mut mid);
      tmpl.categorize_witness_terms(&gate.b, &mut inputs, &mut mid);
      tmpl.categorize_witness_terms(&gate.c, &mut inputs, &mut mid);
    }

    tmpl.build_witness(f, &inputs, &mid);

    let vec_size = &tmpl.witness.len();

    // create a, b anc c vectors for each gate
    for gate in gates {
      let mut a = SparseVec::new(f, vec_size);
      tmpl.build_constraint_vec(f, &mut a, &gate.a);

      let mut b = SparseVec::new(f, vec_size);
      tmpl.build_constraint_vec(f, &mut b, &gate.b);

      let mut c = SparseVec::new(f, vec_size);
      tmpl.build_constraint_vec(f, &mut c, &gate.c);

      let constraint = Constraint { a, b, c };
      tmpl.constraints.push(constraint)
    }
    tmpl
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::zk::w_trusted_setup::pinocchio::equation_parser::EquationParser;

  #[test]
  fn test_constraint_generation() {
    let f = &PrimeField::new(&3911u16);
    {
      // Num
      let mut tmpl = R1CSTmpl::new(f);
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::Num(f.elem(&4u8));
      tmpl.categorize_witness_terms(term, &mut inputs, &mut mid);
      tmpl.build_witness(f, &inputs, &mid);

      let mut constraint = SparseVec::new(f, &2u8);
      tmpl.build_constraint_vec(f, &mut constraint, &term);

      // should be mapped to One term at index 0
      assert_eq!(constraint.get(&0u8), &f.elem(&4u8));
    }
    {
      // Sum
      let mut tmpl = R1CSTmpl::new(f);
      let mut inputs = vec![];
      let mut mid = vec![];

      let y = Term::Var("y".to_string());
      let z = Term::Var("z".to_string());
      let term = &Term::Sum(Box::new(y.clone()), Box::new(z.clone()));
      tmpl.categorize_witness_terms(term, &mut inputs, &mut mid);
      tmpl.build_witness(f, &inputs, &mid);

      let mut constraint = SparseVec::new(f, &3u8);
      tmpl.build_constraint_vec(f, &mut constraint, &term);

      // y and z should be stored at index 1 and 2 of witness vector respectively
      assert_eq!(constraint.get(&1u8), &f.elem(&1u8));
      assert_eq!(constraint.get(&1u8), &f.elem(&1u8));
    }
  }

  #[test]
  fn test_categorize_witness_terms() {
    let f = &PrimeField::new(&3911u16);

    // Num term should not be categorized as input or mid
    {
      let tmpl = R1CSTmpl::new(f);
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::Num(f.elem(&9u8));
      tmpl.categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 0);
      assert_eq!(mid.len(), 0);
    }

    // One term should not be categorized as input or mid
    {
      let tmpl = R1CSTmpl::new(f);
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::One;
      tmpl.categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 0);
      assert_eq!(mid.len(), 0);
    }

    // Var term should be categorized as input
    {
      let tmpl = R1CSTmpl::new(f);
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::Var("x".to_string());
      tmpl.categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 1);
      assert_eq!(mid.len(), 0);
    }

    // Out term should be not categorized as input or mid
    {
      let tmpl = R1CSTmpl::new(f);
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::Out;
      tmpl.categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 0);
      assert_eq!(mid.len(), 0);
    }

    // TmpVar term should be categorized as mid
    {
      let tmpl = R1CSTmpl::new(f);
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::TmpVar(1);
      tmpl.categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 0);
      assert_eq!(mid.len(), 1);
    }

    // Sum term should be recursively categorized
    {
      let tmpl = R1CSTmpl::new(f);
      let mut inputs = vec![];
      let mut mid = vec![];
      let y = Term::Var("y".to_string());
      let z = Term::Var("z".to_string());
      let term = &Term::Sum(Box::new(y.clone()), Box::new(z.clone()));
      tmpl.categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 2);
      assert_eq!(mid.len(), 0);
    }
  }

  #[test]
  fn test_build_witness() {
    let f = &PrimeField::new(&3911u16);
    let mut tmpl = R1CSTmpl::new(f);
    assert_eq!(tmpl.indices.len(), 1);

    // initially witness contains only One term
    assert_eq!(tmpl.indices.get(&Term::One).unwrap(), &f.elem(&0u8));
    assert_eq!(tmpl.witness.len(), 1);
    assert_eq!(tmpl.witness[0], Term::One);

    let a = Term::Var("a".to_string());
    let b = Term::Var("b".to_string());
    let sum = Term::Sum(Box::new(a), Box::new(b));

    let terms = vec![
      Term::Num(f.elem(&9u8)),  // Num should be ignored
      Term::One,  // One should not be added twice
      Term::Var("x".to_string()),  // Var should be added
      Term::Var("x".to_string()),  // the same Var should be added twice
      Term::Var("y".to_string()),  // different Var should be added
      Term::TmpVar(1),  // TmpVar should be added
      Term::TmpVar(1),  // same TmpVar should not be added twice
      Term::TmpVar(2),  // different TmpVar should be added
      Term::Out,  // Out should be added
      Term::Out,  // Out should not be added twice
      sum,  // sum should be added recursively
    ];

    let mut inputs = vec![];
    let mut mid = vec![];

    for term in &terms {
      tmpl.categorize_witness_terms(term, &mut inputs, &mut mid);
    }
    tmpl.build_witness(f, &inputs, &mid);
    assert_eq!(tmpl.indices.len(), 8);
    assert_eq!(tmpl.witness.len(), 8);

    let exp = vec![
      Term::One,
      Term::Var("x".to_string()),
      Term::Var("y".to_string()),
      Term::Var("a".to_string()),
      Term::Var("b".to_string()),
      Term::Out,
      Term::TmpVar(1),
      Term::TmpVar(2),
    ];
    assert!(tmpl.witness == exp);
  }

  #[test]
  fn test_witness_indices() {
    let f = &PrimeField::new(&3911u16);
    let input = "(3 * x + 4) / 2 == 11";
    let eq = EquationParser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let r1cs = R1CSTmpl::from_gates(f, gates);

    let h = r1cs.indices;
    let w = [
      Term::One,
      Term::Var("x".to_string()),
      Term::Out,
      Term::TmpVar(1),
      Term::TmpVar(2),
      Term::TmpVar(3),
    ];
    assert_eq!(h.len(), w.len());

    for (i, term) in w.iter().enumerate() {
      assert_eq!(h.get(&term).unwrap(), &f.elem(&i));
    }
  }

  fn term_to_str(tmpl: &R1CSTmpl, vec: &SparseVec) -> String {
    let mut indices = vec.indices().to_vec();
    indices.sort();  // sort to make indices order deterministic
    let s = indices.iter().map(|i| {
      let i_usize: usize = i.e.clone().try_into().unwrap();
      match &tmpl.witness[i_usize] {
        Term::Var(s) => s.clone(),
        Term::TmpVar(i) => format!("t{}", i),
        Term::One => format!("{:?}", &vec.get(i).e),
        Term::Out => "out".to_string(),
        // currently not handling Term::Sum since it's not used in tests
        _ => "?".to_string(),
      }
    }).collect::<Vec<String>>().join(" + ");
    format!("{}", s)
  }

  #[test]
  fn test_r1cs_build_a_b_c_matrix() {
    let f = &PrimeField::new(&3911u16);
    let input = "3 * x + 4 == 11";
    let eq = EquationParser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = R1CSTmpl::from_gates(f, gates);

    let mut res = vec![];
    for constraint in &tmpl.constraints {
      let a = term_to_str(&tmpl, &constraint.a);
      let b = term_to_str(&tmpl, &constraint.b);
      let c = term_to_str(&tmpl, &constraint.c);
      res.push((a, b, c));
    }

    assert_eq!(res.len(), 3);
    assert_eq!(res[0], ("3".to_string(), "x".to_string(), "t1".to_string()));
    assert_eq!(res[1], ("4 + t1".to_string(), "1".to_string(), "t2".to_string()));
    assert_eq!(res[2], ("t2".to_string(), "1".to_string(), "out".to_string()));
  }

  #[test]
  fn blog_post_1_example_1() {
    let f = &PrimeField::new(&37u8);
    let expr = "(x * x * x) + x + 5 == 35";
    let eq = EquationParser::parse(f, expr).unwrap();
    let gates = &Gate::build(f, &eq);
    let r1cs_tmpl = R1CSTmpl::from_gates(f, gates);

    println!("{:?}", r1cs_tmpl.witness);
  }

  #[test]
  fn blog_post_1_example_2() {
    let f = &PrimeField::new(&37u8);
    let expr = "(x * x * x) + x + 5 == 35";
    let eq = EquationParser::parse(f, expr).unwrap();
    let gates = &Gate::build(f, &eq);
    let r1cs_tmpl = R1CSTmpl::from_gates(f, gates);

    println!("w = {:?}", r1cs_tmpl.witness);
    println!("{:?}", r1cs_tmpl.constraints);
  }
}
