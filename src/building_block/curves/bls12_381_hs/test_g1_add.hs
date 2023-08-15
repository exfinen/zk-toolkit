import Pairing_bls12381

get_pt :: Point Fq1 -> (Integer, Integer)
get_pt (Affine {ax = Fq1 {t0 = x}, ay = Fq1 {t0 = y}}) = (x, y)
get_pt (PointAtInfinity) = (0, 0)

g1_show :: Integer -> (Point Fq1) -> IO ()
g1_show i p = do
  putStr "Xy { x: b\""
  let (x, y) = get_pt p
  putStr (show x)
  putStr "\", y: b\""
  putStr (show y)
  putStrLn "\" },"

do_it :: Integer -> Maybe (Point Fq1) -> IO ()
do_it i p = do
  if i <= 10 then do
    case p of
      Just p2 -> do
        g1_show i p2
        case g1Generator of
          Just g -> do
            let p3 = pointAdd p2 g
            do_it (i + 1) (Just p3)
          Nothing -> putStrLn "failed"
      Nothing -> putStrLn "failed"
  else return ()

main = do
  let i = 1
  let g = g1Generator
  do_it i g
