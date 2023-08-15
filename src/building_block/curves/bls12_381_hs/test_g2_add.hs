import Pairing_bls12381
import Data.Maybe (fromJust)

get_pt :: Point Fq2 -> (Integer, Integer, Integer, Integer)
get_pt (Affine (Fq2 (Fq1 x1) (Fq1 x0)) (Fq2 (Fq1 y1) (Fq1 y0))) = (x1, x0, y1, y0)
get_pt (PointAtInfinity) = (0, 0, 0, 0)

pt_show :: Point Fq2 -> IO ()
pt_show p = do
  let (x1, x0, y1, y0) = get_pt p
  putStr "Xy { "
  putStr $ "x1: b\"" ++ show x1 ++ "\", "
  putStr $ "x0: b\"" ++ show x0 ++ "\", "
  putStr $ "y1: b\"" ++ show y1 ++ "\", "
  putStr $ "y0: b\"" ++ show y0 ++ "\", "
  putStrLn "}, "

loop :: Integer -> Point Fq2 -> Point Fq2 -> IO ()
loop i g p = do
  if i > 0 then do
    pt_show p
    loop (i-1) g (pointAdd p g)
  else return ()

main = do
  let g = fromJust g2Generator
  loop 10 g g
