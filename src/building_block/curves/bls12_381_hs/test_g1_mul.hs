import Pairing_bls12381

get_pt :: Point Fq1 -> (Integer, Integer)
get_pt (Affine {ax = Fq1 {t0 = x}, ay = Fq1 {t0 = y}}) = (x, y)
get_pt (PointAtInfinity) = (0, 0)

g1_show :: Maybe (Point Fq1) -> Integer -> IO ()
g1_show p m = do
  case p of
    Just p2 -> do
      putStr "ScalarMulTest { x: b\""
      let (x, y) = get_pt p2
      putStr (show x)
      putStr "\", y: b\""
      putStr (show y)
      putStr "\", multiple: b\""
      putStr (show m)
      putStrLn "\" },"
    Nothing -> putStrLn "g1_show failed"

do_it :: Integer -> Integer -> Maybe (Point Fq1) -> IO ()
do_it i m p = do
  if i <= 1 then do
    case p of
      Just p2 -> do
        case g1Generator of
          Just g -> do
            let p3 = pointMul m g
            g1_show p3 m
            do_it (i + 1) m p3
          Nothing -> putStrLn "failed"
      Nothing -> putStrLn "failed"
  else return ()

main = do
  let i = 1
  let g = g1Generator
  do_it i 123456789012345678901234567890 g
