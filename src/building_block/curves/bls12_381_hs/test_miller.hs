import Pairing_bls12381
import Data.Bits (shiftR)
import Data.List (unfoldr)

miller :: () -> IO ()
miller () = miller' iterations
  where
    iterations = tail $ reverse $
      unfoldr (\b -> if b == (0 :: Integer) then Nothing
                     else Just(odd b, shiftR b 1)) 0xd201000000010000

miller' :: [Bool] -> IO ()
miller' [] = putStr "\n"
miller' (i:iters) =
  if i
    then do
      putStr "1"
      miller' iters
    else do 
      putStr "0"
      miller' iters

main = do
  miller ()

