module Common (getFq1Values, getFq2Values, getFq6Values) where

import Pairing_bls12381

getFq1Values   :: () -> (Fq1, Fq1, Fq1, Fq1)
getFq1Values () = do
  let a1 = negate $ Fq1 3
  let b1 = negate $ Fq1 5
  let c1 = negate $ Fq1 7
  let d1 = negate $ Fq1 9
  (a1, b1, c1, d1)

getFq2Values   :: () -> (Fq2, Fq2, Fq2, Fq2)
getFq2Values () = do
  let (a1, b1, c1, d1) = getFq1Values ()
  let a2 = Fq2 a1 b1
  let b2 = Fq2 b1 c1
  let c2 = Fq2 c1 d1
  let d2 = Fq2 d1 a1
  (a2, b2, c2, d2)

getFq6Values   :: () -> (Fq6, Fq6, Fq6, Fq6)
getFq6Values () = do
  let (a2, b2, c2, d2) = getFq2Values ()
  let a6 = Fq6 a2 b2 c2
  let b6 = Fq6 b2 c2 d2
  let c6 = Fq6 c2 d2 a2
  let d6 = Fq6 d2 a2 b2
  (a6, b6, c6, d6)
