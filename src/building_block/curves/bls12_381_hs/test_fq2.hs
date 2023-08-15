import Pairing_bls12381
import Common

fq2_add () = do
  let (a1, b1, c1, d1) = getFq1Values ()
  let a2 = Fq2 a1 b1
  let b2 = Fq2 c1 d1
  let x = a2 + b2

  putStr "Fq2 Add: "
  print x

fq2_sub () = do
  let (a1, b1, c1, d1) = getFq1Values ()
  let a2 = Fq2 a1 b1
  let b2 = Fq2 c1 d1
  let x = a2 - b2

  putStr "Fq2 Sub: "
  print x

fq2_mul () = do
  let (a1, b1, c1, d1) = getFq1Values ()
  let a2 = Fq2 a1 b1
  let b2 = Fq2 c1 d1
  let x = a2 * b2

  putStr "Fq2 Mul: "
  print x

fq2_inv () = do
  let (a1, b1, c1, d1) = getFq1Values ()
  let a2 = Fq2 a1 b1
  let b2 = Fq2 c1 d1
  let x = inv a2
  putStr "Fq2 Mul: "
  print x
  let x = inv b2
  putStr "Fq2 Mul: "
  print x

fq2_mul_nonres () = do
  let (a1, b1, c1, d1) = getFq1Values ()
  let a2 = Fq2 a1 b1
  let b2 = Fq2 c1 d1
  putStr "Fq2 MulNonres: "
  let x = a2 * b2
  print x
  let x = mul_nonres (a2 * b2)
  print x

main = do
  fq2_add ()
  -- fq2_sub ()
  -- fq2_mul ()
  -- fq2_inv ()
  -- fq2_mul_nonres ()