import Pairing_bls12381
import Data.Maybe (fromJust)

get_pt :: Point Fq1 -> (Integer, Integer)
get_pt (Affine (Fq1 x) (Fq1 y)) = (x, y)
get_pt (PointAtInfinity) = (0, 0)

pt_show :: Point Fq1 -> IO ()
pt_show p = do
  let (x, y) = get_pt p
  putStr "Xy { "
  putStr $ "x: b\"" ++ show x ++ "\", "
  putStr $ "y: b\"" ++ show y ++ "\", "
  putStrLn "}, "

loop :: Integer -> Point Fq1 -> Point Fq1 -> IO ()
loop i p g = do
  if i > 0 then do
    pt_show p
    loop (i-1) (pointAdd p g) g
  else return ()

main = do
  let g = fromJust g1Generator
  loop 10 g g
