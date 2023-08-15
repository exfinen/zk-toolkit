import Pairing_bls12381
import Common

fq12_add () = do
  let (a6, b6, c6, d6) = getFq6Values ()
  let a12 = Fq12 a6 b6
  let b12 = Fq12 c6 d6
  let x = a12 + b12
  putStr "Fq12 Add: "
  print x

fq12_sub () = do
  let (a6, b6, c6, d6) = getFq6Values ()
  let a12 = Fq12 a6 b6
  let b12 = Fq12 c6 d6
  let x = a12 - b12
  putStr "Fq12 Sub: "
  print x

fq12_mul () = do
  let (a6, b6, c6, d6) = getFq6Values ()
  let a12 = Fq12 a6 b6
  let b12 = Fq12 c6 d6
  let x = a12 * b12
  putStr "Fq12 Mul: "
  print x

fq12_inv () = do
  let (a6, b6, c6, d6) = getFq6Values ()
  let a12 = Fq12 a6 b6
  let b12 = Fq12 c6 d6
  let x = inv a12
  putStr "Fq12 Inv1: "
  print x
  let x = inv b12
  putStr "Fq12 Inv2: "
  print x

main = do
  -- fq12_add ()
  -- fq12_sub ()
  -- fq12_mul ()
  fq12_inv ()