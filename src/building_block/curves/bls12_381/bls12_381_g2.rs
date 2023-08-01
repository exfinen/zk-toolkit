#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct BLS12_381_G2 {
}

/*
pub static G2_GENERATOR: Lazy<G2Point> = Lazy::new(|| {
  let x1 = BigUint::parse_bytes(b"13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e", 16).unwrap();
  let x2 = BigUint::parse_bytes(b"024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8", 16).unwrap();
  let x = Fq2::new(
    &Fq1::new(&BASE_FIELD, &x1),
    &Fq1::new(&BASE_FIELD, &x2),
  );
  let y1 = BigUint::parse_bytes(b"0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be", 16).unwrap();
  let y2 = BigUint::parse_bytes(b"0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801", 16).unwrap();
  let y = Fq2::new(
    &Fq1::new(&BASE_FIELD, &y1),
    &Fq1::new(&BASE_FIELD, &y2),
  );
  G2Point { x, y }
});
*/