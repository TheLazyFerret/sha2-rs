//! Author: TheLazyFerret (https://github.com/TheLazyFerret)
//! Copyright (c) 2025 TheLazyFerret
//! Licensed under the MIT License. See LICENSE file in the project root for full license information.
//!
//! sha256 implementation

use std::fmt;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufReader, Read};

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

const KINITHASH: [u32; 8] = [
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const U32SIZE: usize = u32::BITS as usize;

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
  right_rotation(x, 2) ^ right_rotation(x, 13) ^ right_rotation(x, 22)
}

#[inline]
fn upper_sigma1(x: u32) -> u32 {
  right_rotation(x, 6) ^ right_rotation(x, 11) ^ right_rotation(x, 25)
}

#[inline]
fn lower_sigma0(x: u32) -> u32 {
  right_rotation(x, 7) ^ right_rotation(x, 18) ^ (x >> 3)
}

#[inline]
fn lower_sigma1(x: u32) -> u32 {
  right_rotation(x, 17) ^ right_rotation(x, 19) ^ (x >> 10)
}

#[inline]
fn calculate_schedule(message: &[u32; 64], i: usize) -> u32 {
  assert!(i > 15 && i < 64);
  lower_sigma1(message[i - 2])
    .wrapping_add(message[i - 7])
    .wrapping_add(lower_sigma0(message[i - 15]))
    .wrapping_add(message[i - 16])
}

// Calculate the value of T1
#[inline]
fn calculate_t1(message: &[u32; 64], hash: &[u32; 8], iteration: usize) -> u32 {
  assert!(iteration < 64);
  hash[7]
    .wrapping_add(upper_sigma1(hash[4]))
    .wrapping_add(choose(hash[4], hash[5], hash[6]))
    .wrapping_add(KCONSTANTS[iteration])
    .wrapping_add(message[iteration])
}

// Calculate the value of T2
#[inline]
fn calculate_t2(hash: &[u32; 8], iteration: usize) -> u32 {
  assert!(iteration < 64);
  upper_sigma0(hash[0]).wrapping_add(majority(hash[0], hash[1], hash[2]))
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// auxiliar struct that holds all the params used in the iteration for parsing the blocks.
struct Sha256Iter {
  bytes_readed: u64,
  pad: bool,
  finished: bool,
  buffer: [u8; 64],
}

impl Default for Sha256Iter {
  fn default() -> Self {
    Sha256Iter {
      bytes_readed: 0,
      pad: false,
      finished: false,
      buffer: [0; 64],
    }
  }
}

impl Sha256Iter {
  /// Creates a new instance of the struct
  pub fn new() -> Self {
    Sha256Iter::default()
  }

  /// Parse the buffer into a block
  pub fn parse(&self) -> [u32; 16] {
    let mut x: [u32; 16] = [0; 16];
    for n in x.iter_mut().enumerate() {
      let bytes: [u8; 4] = self.buffer[(n.0 * 4)..(n.0 * 4 + 4)].try_into().unwrap();
      *n.1 = u32::from_be_bytes(bytes);
    }
    x
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Main struct, representing a hash
pub struct Sha256 {
  hash: [u32; 8],
}

/// impl for printing, also derives into to_string
impl fmt::Display for Sha256 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut a = String::new();
    for byte in self.hash {
      let mut aux = String::new();
      write!(&mut aux, "{:0<8x}", byte).expect("error formatting");
      a.push_str(&aux);
    }
    write!(f, "{a}")
  }
}

impl Sha256 {
  /// initialize the struct
  fn init() -> Self {
    Sha256 { hash: KINITHASH }
  }

  /// "Main function" that get the sha256 from a file descriptor
  pub fn from_file(file: File) -> Self {
    let mut hash = Sha256::init();
    let mut bufre = BufReader::with_capacity(1048576, file); // 500mb
    let mut iter = Sha256Iter::new();
    while !iter.finished {
      Sha256::get_next_block(&mut bufre, &mut iter);
      Sha256::compute_block(&mut hash, &iter.parse());
    }
    hash
  }

  /// "Main function" that get the sha256 from an slice of bytes
  pub fn from_bytes(bytes: &[u8]) -> Self {
    let mut hash = Sha256::init();
    let mut bufre = BufReader::new(bytes);
    let mut iter = Sha256Iter::new();
    while !iter.finished {
      Sha256::get_next_block(&mut bufre, &mut iter);
      Sha256::compute_block(&mut hash, &iter.parse());
    }
    hash
  }

  /// Load a new unparsed block into the auxiliar iteration struct
  fn get_next_block<T: std::io::Read>(bufre: &mut BufReader<T>, iter: &mut Sha256Iter) {
    iter.buffer = [0; 64];
    let bytes_readed =
      Read::read(bufre, &mut iter.buffer).expect("unexpected error reading the file");
    iter.bytes_readed += bytes_readed as u64;
    // last block
    if bytes_readed < 56 {
      if !iter.pad {
        iter.buffer[bytes_readed] = 0x80;
        iter.pad = true;
      }
      iter.buffer[56..=63].clone_from_slice(&(iter.bytes_readed * 8).to_be_bytes());
      iter.finished = true;
    }
    // one block left
    else if bytes_readed < 64 && !iter.pad {
      iter.buffer[bytes_readed] = 0x80;
      iter.pad = true;
    }
    // any ammount of blocks left
  }

  /// Compute the block, return the updated hash value in each iteration
  fn compute_block(hash: &mut Sha256, block: &[u32; 16]) {
    let mut message: [u32; 64] = [0; 64];
    message[0..16].copy_from_slice(block);
    // Message schedule
    for n in 16..64 {
      message[n] = calculate_schedule(&message, n);
    }
    let mut aux_hash = hash.hash; // copy
    // compute working variables
    for n in 0..64 {
      let t1 = calculate_t1(&message, &aux_hash, n);
      let t2 = calculate_t2(&aux_hash, n);
      aux_hash[7] = aux_hash[6];
      aux_hash[6] = aux_hash[5];
      aux_hash[5] = aux_hash[4];
      aux_hash[4] = aux_hash[3].wrapping_add(t1);
      aux_hash[3] = aux_hash[2];
      aux_hash[2] = aux_hash[1];
      aux_hash[1] = aux_hash[0];
      aux_hash[0] = t1.wrapping_add(t2);
    }
    // compute intermedial hash
    hash.hash[0] = aux_hash[0].wrapping_add(hash.hash[0]);
    hash.hash[1] = aux_hash[1].wrapping_add(hash.hash[1]);
    hash.hash[2] = aux_hash[2].wrapping_add(hash.hash[2]);
    hash.hash[3] = aux_hash[3].wrapping_add(hash.hash[3]);
    hash.hash[4] = aux_hash[4].wrapping_add(hash.hash[4]);
    hash.hash[5] = aux_hash[5].wrapping_add(hash.hash[5]);
    hash.hash[6] = aux_hash[6].wrapping_add(hash.hash[6]);
    hash.hash[7] = aux_hash[7].wrapping_add(hash.hash[7]);
  }
}
