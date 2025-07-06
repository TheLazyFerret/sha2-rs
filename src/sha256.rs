//! Author: TheLazyFerret (https://github.com/TheLazyFerret)
//! Copyright (c) 2025 TheLazyFerret
//! Licensed under the MIT License. See LICENSE file in the project root for full license information.
//!
//! sha256 implementation

const KCONSTANTS: [u32; 64] = [
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const INITHASH: [u32; 8] = [
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const U32SIZE: usize = size_of::<u32>() * 8;

#[inline]
fn right_rotation(number: u32, n: usize) -> u32 {
  assert!(n < U32SIZE);
  (number >> n) | (number << (U32SIZE - n))
}

#[inline]
fn choose(x: u32, y: u32, z: u32) -> u32 {
  (x & y) ^ (!x & z)
}

#[inline]
fn majority(x: u32, y: u32, z: u32) -> u32 {
  (x & y) ^ (x & z) ^ (y & z)
}

#[inline]
fn upper_sigma0(x: u32) -> u32 {
  BasicFunctions::right_rotation(&x, 2)
    ^ BasicFunctions::right_rotation(&x, 13)
    ^ BasicFunctions::right_rotation(&x, 22)
}

#[inline]
fn upper_sigma1(x: u32) -> u32 {
  BasicFunctions::right_rotation(&x, 6)
    ^ BasicFunctions::right_rotation(&x, 11)
    ^ BasicFunctions::right_rotation(&x, 25)
}

#[inline]
fn lower_sigma0(x: u32) -> u32 {
  BasicFunctions::right_rotation(&x, 7)
    ^ BasicFunctions::right_rotation(&x, 18)
    ^ BasicFunctions::right_shift(&x, 3)
}

#[inline]
fn lower_sigma1(x: u32) -> u32 {
  BasicFunctions::right_rotation(&x, 17)
    ^ BasicFunctions::right_rotation(&x, 19)
    ^ BasicFunctions::right_shift(&x, 10)
}

// Calculate the value of T1
fn calculate_t1(message: &[u32; 64], hash: &[u32; 8], iteration: usize) -> u32 {
  assert!(iteration < 64);
  let x: u32 = {
    hash[7]
      .wrapping_add(upper_sigma1(hash[4]))
      .wrapping_add(choose(hash[4], hash[5], hash[6]))
      .wrapping_add(KCONSTANTS[iteration])
      .wrapping_add(message[iteration])
  };
  return x;
}

// Calculate the value of T2
fn calculate_t2(hash: &[u32; 8], iteration: usize) -> u32 {
  assert!(iteration < 64);
  let x: u32 = { upper_sigma0(hash[0]).wrapping_add(majority(hash[0], hash[1], hash[2])) };
  return x;
}
