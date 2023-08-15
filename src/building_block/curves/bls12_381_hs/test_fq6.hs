import Pairing_bls12381
import Common

fq6_add () = do
  let (a2, b2, c2, d2) = getFq2Values ()
  let a6 = Fq6 a2 b2 c2
  let b6 = Fq6 b2 c2 d2
  let x = a6 + b6

  putStr "Fq6 Add: "
  print x

fq6_sub () = do
  let (a2, b2, c2, d2) = getFq2Values ()
  let a6 = Fq6 a2 b2 c2
  let b6 = Fq6 b2 c2 d2
  let x = a6 - b6

  putStr "Fq6 Sub: "
  print x

fq6_mul () = do
  let (a2, b2, c2, d2) = getFq2Values ()
  let a6 = Fq6 a2 b2 c2
  let b6 = Fq6 b2 c2 d2
  let x = a6 * b6

  putStr "Fq6 Mul: "
  print x

fq6_inv () = do
  let (a2, b2, c2, d2) = getFq2Values ()
  let a6 = Fq6 a2 b2 c2
  let b6 = Fq6 b2 c2 d2
  let x = inv a6
  putStr "Fq6 Inv1: "
  print x
  let x = inv b6
  putStr "Fq6 Inv2: "
  print x

fq6_mul_nonres () = do
  let (a2, b2, c2, d2) = getFq2Values ()
  let a6 = Fq6 a2 b2 c2
  let b6 = Fq6 b2 c2 d2
  putStr "Fq6 MulNonres: "
  let x = a6 * b6
  print x
  let x = mul_nonres (a6 * b6)
  print x

main = do
  -- fq6_add ()
  -- fq6_sub ()
  -- fq6_mul ()
  -- fq6_inv ()
  fq6_mul_nonres ()