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

main = do
  let g = fromJust g1Generator
  let p = fromJust (pointMul 123456789012345678901234567890 g)
  pt_show p

