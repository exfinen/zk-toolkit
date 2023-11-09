use crate::building_block::mcl::{
  mcl_fr::MclFr,
  mcl_sparse_vec::MclSparseVec,
  qap::{
    term::Term,
    gate::Gate,
    constraint::Constraint,
  },
};
use std::collections::HashMap;
use num_traits::Zero;

pub struct R1CSTmpl {
  pub constraints: Vec<Constraint>,
  pub witness: Vec<Term>,
  pub indices: HashMap<Term, MclFr>,
  pub mid_beg: MclFr,
}

impl R1CSTmpl {
  // build witness vector whose elements in the following order:
  // 1, inputs, Out, mid
  fn build_witness(
    inputs: &Vec<Term>,
    mid: &Vec<Term>,
    witness: &mut Vec<Term>,
    indices: &mut HashMap::<Term, MclFr>,
  ) -> MclFr {
    let mut i = MclFr::from(1);  // `1` has already been added in new function

    for x in inputs {
      witness.push(x.clone());
      indices.insert(x.clone(), i.clone());
      i.inc();
    }
    witness.push(Term::Out);
    indices.insert(Term::Out, i.clone());
    i.inc();

    let mid_beg = i.clone();

    for x in mid {
      witness.push(x.clone());
      indices.insert(x.clone(), i.clone());
      i.inc();
    }

    mid_beg
  }

  fn categorize_witness_terms(
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
        R1CSTmpl::categorize_witness_terms(&a, inputs, mid);
        R1CSTmpl::categorize_witness_terms(&b, inputs, mid);
      },
    }
  }

  fn build_constraint_vec(
    vec: &mut MclSparseVec,
    term: &Term,
    indices: &HashMap::<Term, MclFr>,
  ) {
    match term {
      Term::Sum(a, b) => {
        R1CSTmpl::build_constraint_vec(vec, &a, indices);
        R1CSTmpl::build_constraint_vec(vec, &b, indices);
      },
      Term::Num(n) => {
        vec.set(&MclFr::zero(), n);  // Num is represented as Term::One at index 0 times n
      },
      x => {
        let index = indices.get(&x).unwrap();
        vec.set(index, &MclFr::from(1));
      },
    }
  }

  pub fn new(gates: &[Gate]) -> Self {
    let mut witness = vec![];
    let mut indices = HashMap::<Term, MclFr>::new();

    // add `1` at index 0
    witness.push(Term::One);
    indices.insert(Term::One, MclFr::zero());

    // categoraize terms contained in gates to inputs and mid
    let mut inputs = vec![];
    let mut mid = vec![];

    for gate in gates {
      R1CSTmpl::categorize_witness_terms(&gate.a, &mut inputs, &mut mid);
      R1CSTmpl::categorize_witness_terms(&gate.b, &mut inputs, &mut mid);
      R1CSTmpl::categorize_witness_terms(&gate.c, &mut inputs, &mut mid);
    }

    let mid_beg = R1CSTmpl::build_witness(&inputs, &mid, &mut witness, &mut indices);  
    let vec_size = &MclFr::from(witness.len());
    let mut constraints = vec![];

    // create a, b anc c vectors for each gate
    for gate in gates {
      let mut a = MclSparseVec::new(vec_size);
      R1CSTmpl::build_constraint_vec(&mut a, &gate.a, &indices);

      let mut b = MclSparseVec::new(vec_size);
      R1CSTmpl::build_constraint_vec(&mut b, &gate.b, &indices);

      let mut c = MclSparseVec::new(vec_size);
      R1CSTmpl::build_constraint_vec(&mut c, &gate.c, &indices);

      let constraint = Constraint { a, b, c };
      constraints.push(constraint)
    }

    R1CSTmpl {
      constraints,
      witness,
      indices,
      mid_beg,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::mcl::{
    qap::equation_parser::EquationParser,
    mcl_initializer::MclInitializer,
  };

  #[test]
  fn test_categorize_witness_terms() {
    MclInitializer::init();
    // Num term should not be categorized as input or mid
    {
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::Num(MclFr::from(9));
      R1CSTmpl::categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 0);
      assert_eq!(mid.len(), 0);
    }

    // One term should not be categorized as input or mid
    {
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::One;
      R1CSTmpl::categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 0);
      assert_eq!(mid.len(), 0);
    }

    // Var term should be categorized as input
    {
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::Var("x".to_string());
      R1CSTmpl::categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 1);
      assert_eq!(mid.len(), 0);
    }

    // Out term should be not categorized as input or mid
    {
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::Out;
      R1CSTmpl::categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 0);
      assert_eq!(mid.len(), 0);
    }

    // TmpVar term should be categorized as mid
    {
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::TmpVar(1);
      R1CSTmpl::categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 0);
      assert_eq!(mid.len(), 1);
    }

    // Sum term should be recursively categorized
    {
      let mut inputs = vec![];
      let mut mid = vec![];
      let y = Term::Var("y".to_string());
      let z = Term::Var("z".to_string());
      let term = &Term::Sum(Box::new(y.clone()), Box::new(z.clone()));
      R1CSTmpl::categorize_witness_terms(term, &mut inputs, &mut mid);
      assert_eq!(inputs.len(), 2);
      assert_eq!(mid.len(), 0);
    }
  }

  #[test]
  fn test_build_witness() {
    MclInitializer::init();
    let a = Term::Var("a".to_string());
    let b = Term::Var("b".to_string());
    let sum = Term::Sum(Box::new(a), Box::new(b));

    let terms = vec![
      Term::Num(MclFr::from(9)),  // Num should be ignored
      Term::One,  // One is discarded in categorize_witness_terms
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
      R1CSTmpl::categorize_witness_terms(term, &mut inputs, &mut mid);
    }

    let mut witness = vec![];
    let mut indices = HashMap::<Term, MclFr>::new();

    let mid_beg = R1CSTmpl::build_witness(&inputs, &mid, &mut witness, &mut indices);
    assert!(mid_beg == MclFr::from(6));

    // 7 since One has been discarded and Out is added in build_witness
    assert_eq!(indices.len(), 7);
    assert_eq!(witness.len(), 7);

    // check if witness is correctly built
    let exp = vec![
      // One has been discarded
      Term::Var("x".to_string()),
      Term::Var("y".to_string()),
      Term::Var("a".to_string()),
      Term::Var("b".to_string()),
      Term::Out,  // build_witness adds Out
      Term::TmpVar(1),
      Term::TmpVar(2),
    ];
    assert!(witness == exp);

    // check if indices map is correctly built
    assert!(indices.get(&Term::One).is_none());
    assert!(indices.get(&Term::Var("x".to_string())).unwrap() == &MclFr::from(1));
    assert!(indices.get(&Term::Var("y".to_string())).unwrap() == &MclFr::from(2));
    assert!(indices.get(&Term::Var("a".to_string())).unwrap() == &MclFr::from(3));
    assert!(indices.get(&Term::Var("b".to_string())).unwrap() == &MclFr::from(4));
    assert!(indices.get(&Term::Out).unwrap() == &MclFr::from(5));
    assert!(indices.get(&Term::TmpVar(1)).unwrap() == &MclFr::from(6));
    assert!(indices.get(&Term::TmpVar(2)).unwrap() == &MclFr::from(7));
  }

  #[test]
  fn test_new() {
    MclInitializer::init();
    let gates = vec![];
    let tmpl = R1CSTmpl::new( &gates);
    assert_eq!(tmpl.indices.len(), 2);

    // if gates is empty, witness should contain only One term and Out term
    assert_eq!(tmpl.indices.get(&Term::One).unwrap(), &MclFr::from(0));
    assert_eq!(tmpl.indices.get(&Term::Out).unwrap(), &MclFr::from(1));
    assert_eq!(tmpl.witness.len(), 2);
    assert_eq!(tmpl.witness[0], Term::One);
    assert_eq!(tmpl.witness[1], Term::Out);
  }

  #[test]
  fn test_constraint_generation() {
    MclInitializer::init();
    {
      // Num
      let mut inputs = vec![];
      let mut mid = vec![];
      let term = &Term::Num(MclFr::from(4));
      R1CSTmpl::categorize_witness_terms(term, &mut inputs, &mut mid);

      let mut witness = vec![];
      let mut indices = HashMap::<Term, MclFr>::new();
      let mid_beg = R1CSTmpl::build_witness(&inputs, &mid, &mut witness, &mut indices);
      assert!(mid_beg == MclFr::from(2));

      let mut constraint = MclSparseVec::new(&MclFr::from(2));
      R1CSTmpl::build_constraint_vec(&mut constraint, &term, &indices);

      // should be mapped to One term at index 0
      assert_eq!(constraint.get(&MclFr::zero()), &MclFr::from(4));
    }
    {
      // Sum
      let mut inputs = vec![];
      let mut mid = vec![];

      let y = Term::Var("y".to_string());
      let z = Term::Var("z".to_string());
      let term = &Term::Sum(Box::new(y.clone()), Box::new(z.clone()));
      R1CSTmpl::categorize_witness_terms(term, &mut inputs, &mut mid);

      let mut witness = vec![];
      let mut indices = HashMap::<Term, MclFr>::new();
      let mid_beg = R1CSTmpl::build_witness(&inputs, &mid, &mut witness, &mut indices);
      assert!(mid_beg == MclFr::from(4));

      let mut constraint = MclSparseVec::new(&MclFr::from(3));
      R1CSTmpl::build_constraint_vec(&mut constraint, &term, &indices);

      // y and z should be stored at index 1 and 2 of witness vector respectively
      assert_eq!(constraint.get(&MclFr::from(1)), &MclFr::from(1));
      assert_eq!(constraint.get(&MclFr::from(1)), &MclFr::from(1));
    }
  }


  #[test]
  fn test_witness_indices() {
    MclInitializer::init();
    let input = "(3 * x + 4) / 2 == 11";
    let eq = EquationParser::parse(input).unwrap();

    let gates = &Gate::build(&eq);
    let tmpl = R1CSTmpl::new(gates);

    let h = tmpl.indices;
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
      assert_eq!(h.get(&term).unwrap(), &MclFr::from(i));
    }
  }

  fn term_to_str(tmpl: &R1CSTmpl, vec: &MclSparseVec) -> String {
    MclInitializer::init();
    let mut indices = vec.indices().to_vec();
    indices.sort();  // sort to make indices order deterministic
    let s = indices.iter().map(|i| {
      let i_usize: usize = i.to_usize();
      match &tmpl.witness[i_usize] {
        Term::Var(s) => s.clone(),
        Term::TmpVar(i) => format!("t{}", i),
        Term::One => format!("{:?}", &vec.get(i)),
        Term::Out => "out".to_string(),
        // currently not handling Term::Sum since it's not used in tests
        _ => "?".to_string(),
      }
    }).collect::<Vec<String>>().join(" + ");
    format!("{}", s)
  }

  #[test]
  fn test_r1cs_build_a_b_c_matrix() {
    MclInitializer::init();
    let input = "3 * x + 4 == 11";
    let eq = EquationParser::parse(input).unwrap();

    let gates = &Gate::build(&eq);
    let tmpl = R1CSTmpl::new(gates);

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
    MclInitializer::init();
    let expr = "(x * x * x) + x + 5 == 35";
    let eq = EquationParser::parse(expr).unwrap();
    let gates = &Gate::build(&eq);
    let r1cs_tmpl = R1CSTmpl::new(gates);

    println!("{:?}", r1cs_tmpl.witness);
  }

  #[test]
  fn blog_post_1_example_2() {
    MclInitializer::init();
    let expr = "(x * x * x) + x + 5 == 35";
    let eq = EquationParser::parse(expr).unwrap();
    let gates = &Gate::build(&eq);
    let r1cs_tmpl = R1CSTmpl::new(gates);

    println!("w = {:?}", r1cs_tmpl.witness);
    println!("{:?}", r1cs_tmpl.constraints);
  }
}

