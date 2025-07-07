// Author: TheLazyFerret (https://github.com/TheLazyFerret)
// Copyright (c) 2025 TheLazyFerret
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use crate::basic_functions::BasicFunctions;
use std::fs;
use std::io;

//struct used for send all the parameters needed for building the next block in all iterations
struct IterationBlock {
    bytes_readed: u128,
    bit_padded: bool,
    finished: bool,
    buffer: [u8; 128],
}

impl IterationBlock {
    // initialize a new IterationBlock instance
    fn new() -> Self {
        let iblock: IterationBlock = IterationBlock {
            bytes_readed: 0,
            bit_padded: false,
            finished: false,
            buffer: [0; 128],
        };
        return iblock;
    }

    // parse the block into [u64; 16] (from [u8; 128])
    fn parse_block(&self) -> [u64; 16] {
        let mut x: [u64; 16] = [0; 16];
        let mut iteration = 0;
        while iteration < 16 {
            x[iteration] = fuse_64(&self.buffer[(iteration * 8)..(iteration * 8 + 8)]);
            iteration += 1;
        }
        return x;
    }
} // impl IterationBlock

// fuse an array of 8 u8 into a u64 BIG ENDIAN
fn fuse_64(array: &[u8]) -> u64 {
    assert_eq!(array.len(), 8);
    let mut x: u64 = 0;
    let mut iteration = 0;
    for n in array {
        let mut aux: u64 = *n as u64;
        aux = aux << (56 - (iteration * 8));
        x = x | aux;
        iteration += 1;
    }
    return x;
}

// function to get the next block to compute from a buffer
fn get_next_block<T: std::io::Read>(
    file_buffer: &mut io::BufReader<T>,
    iterator: &mut IterationBlock,
) {
    iterator.buffer = [0; 128]; // reset the buffer
    let n_bytes_readed: usize =
        io::Read::read(file_buffer, &mut iterator.buffer).expect("error reading the buffer");

    iterator.bytes_readed += n_bytes_readed as u128;

    // last block to append
    if n_bytes_readed < 112 {
        // append the 1 (actually 10000000)
        if iterator.bit_padded == false {
            iterator.buffer[n_bytes_readed] = 0x80;
            iterator.bit_padded = true;
        }
        let size: [u8; 16] = fragment_u128(iterator.bytes_readed * 8);
        let mut iter: i8 = 15;
        while iter >= 0 {
            iterator.buffer[(112 + iter) as usize] = size[15 - iter as usize];
            iter -= 1;
        }
        iterator.finished = true;
    }
    // not the last block to append
    else if n_bytes_readed >= 112 && n_bytes_readed < 128 {
        // append the 1
        if iterator.bit_padded == false {
            iterator.buffer[n_bytes_readed] = 0x80;
            iterator.bit_padded = true;
        }
    }
} // fn get_next_block

// split a u128 value into an array [u8; 16] BIG ENDIAN
fn fragment_u128(number: u128) -> [u8; 16] {
    let mut help_bytes: [u8; 16] = [0; 16];
    let mut iteration: i32 = 15;
    while iteration >= 0 {
        help_bytes[iteration as usize] = (number >> (iteration * 8)) as u8;
        iteration -= 1;
    }
    return help_bytes;
}

// main function to calculate the hash of a file
pub fn hash_from_file(file: &fs::File) -> crate::Sha512 {
    let mut buffer_read = io::BufReader::new(file);
    let mut iterator = IterationBlock::new();
    let mut hash: [u64; 8] = INITHASH;
    while iterator.finished == false {
        get_next_block(&mut buffer_read, &mut iterator);
        let parsed_block: [u64; 16] = iterator.parse_block();
        hash = compute_block(&parsed_block, &hash);
    }
    return crate::Sha512 { hash: hash };
}

// main function to calculate the hash of a string
pub fn hash_from_string(string: &str) -> crate::Sha512 {
    let mut buffer_read = io::BufReader::new(io::Cursor::new(string.to_string()));
    let mut iterator = IterationBlock::new();
    let mut hash: [u64; 8] = INITHASH;
    while iterator.finished == false {
        get_next_block(&mut buffer_read, &mut iterator);
        let parsed_block: [u64; 16] = iterator.parse_block();
        hash = compute_block(&parsed_block, &hash);
    }
    return crate::Sha512 { hash: hash };
}

// Compute the block, returning the updated hash in each iteration
fn compute_block(block: &[u64; 16], hash: &[u64; 8]) -> [u64; 8] {
    let mut message: [u64; 80] = [0; 80];
    let mut aux_hash: [u64; 8] = *hash;
    message[0..16].clone_from_slice(&block[0..16]);
    for n in 16..80 {
        message[n] = calculate_schedule(&message, n);
    }
    for n in 0..80 {
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
    let mut new_hash: [u64; 8] = [0; 8];
    new_hash[0] = aux_hash[0].wrapping_add(hash[0]);
    new_hash[1] = aux_hash[1].wrapping_add(hash[1]);
    new_hash[2] = aux_hash[2].wrapping_add(hash[2]);
    new_hash[3] = aux_hash[3].wrapping_add(hash[3]);
    new_hash[4] = aux_hash[4].wrapping_add(hash[4]);
    new_hash[5] = aux_hash[5].wrapping_add(hash[5]);
    new_hash[6] = aux_hash[6].wrapping_add(hash[6]);
    new_hash[7] = aux_hash[7].wrapping_add(hash[7]);

    return new_hash;
} // fn compute_block

// Calculate the value of T1
fn calculate_t1(message: &[u64; 80], hash: &[u64; 8], iteration: usize) -> u64 {
    assert!(iteration < 80);
    let x: u64 = {
        hash[7]
            .wrapping_add(upper_sigma1(hash[4]))
            .wrapping_add(choose(hash[4], hash[5], hash[6]))
            .wrapping_add(KCONSTANTS[iteration])
            .wrapping_add(message[iteration])
    };
    return x;
}

// Calculate the value of T2
fn calculate_t2(hash: &[u64; 8], iteration: usize) -> u64 {
    assert!(iteration < 80);
    let x: u64 = { upper_sigma0(hash[0]).wrapping_add(majority(hash[0], hash[1], hash[2])) };
    return x;
}

// Calculate the values of schedules for 16 <= n <= 79
fn calculate_schedule(message: &[u64; 80], iteration: usize) -> u64 {
    assert!(iteration > 15 && iteration < 80);
    let x: u64 = {
        lower_sigma1(message[iteration - 2])
            .wrapping_add(message[iteration - 7])
            .wrapping_add(lower_sigma0(message[iteration - 15]))
            .wrapping_add(message[iteration - 16])
    };
    return x;
}

// BASIC FUNCTIONS
fn choose(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (!x & z)
}

fn majority(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn upper_sigma0(x: u64) -> u64 {
    BasicFunctions::right_rotation(&x, 28)
        ^ BasicFunctions::right_rotation(&x, 34)
        ^ BasicFunctions::right_rotation(&x, 39)
}

fn upper_sigma1(x: u64) -> u64 {
    BasicFunctions::right_rotation(&x, 14)
        ^ BasicFunctions::right_rotation(&x, 18)
        ^ BasicFunctions::right_rotation(&x, 41)
}

fn lower_sigma0(x: u64) -> u64 {
    BasicFunctions::right_rotation(&x, 1)
        ^ BasicFunctions::right_rotation(&x, 8)
        ^ BasicFunctions::right_shift(&x, 7)
}

fn lower_sigma1(x: u64) -> u64 {
    BasicFunctions::right_rotation(&x, 19)
        ^ BasicFunctions::right_rotation(&x, 61)
        ^ BasicFunctions::right_shift(&x, 6)
}

// First 64 bits of the fractional part of the cube roots of the first eighty prime numbers
const KCONSTANTS: [u64; 80] = [
    0x428a2f98d728ae22,
    0x7137449123ef65cd,
    0xb5c0fbcfec4d3b2f,
    0xe9b5dba58189dbbc,
    0x3956c25bf348b538,
    0x59f111f1b605d019,
    0x923f82a4af194f9b,
    0xab1c5ed5da6d8118,
    0xd807aa98a3030242,
    0x12835b0145706fbe,
    0x243185be4ee4b28c,
    0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f,
    0x80deb1fe3b1696b1,
    0x9bdc06a725c71235,
    0xc19bf174cf692694,
    0xe49b69c19ef14ad2,
    0xefbe4786384f25e3,
    0x0fc19dc68b8cd5b5,
    0x240ca1cc77ac9c65,
    0x2de92c6f592b0275,
    0x4a7484aa6ea6e483,
    0x5cb0a9dcbd41fbd4,
    0x76f988da831153b5,
    0x983e5152ee66dfab,
    0xa831c66d2db43210,
    0xb00327c898fb213f,
    0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2,
    0xd5a79147930aa725,
    0x06ca6351e003826f,
    0x142929670a0e6e70,
    0x27b70a8546d22ffc,
    0x2e1b21385c26c926,
    0x4d2c6dfc5ac42aed,
    0x53380d139d95b3df,
    0x650a73548baf63de,
    0x766a0abb3c77b2a8,
    0x81c2c92e47edaee6,
    0x92722c851482353b,
    0xa2bfe8a14cf10364,
    0xa81a664bbc423001,
    0xc24b8b70d0f89791,
    0xc76c51a30654be30,
    0xd192e819d6ef5218,
    0xd69906245565a910,
    0xf40e35855771202a,
    0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8,
    0x1e376c085141ab53,
    0x2748774cdf8eeb99,
    0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63,
    0x4ed8aa4ae3418acb,
    0x5b9cca4f7763e373,
    0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc,
    0x78a5636f43172f60,
    0x84c87814a1f0ab72,
    0x8cc702081a6439ec,
    0x90befffa23631e28,
    0xa4506cebde82bde9,
    0xbef9a3f7b2c67915,
    0xc67178f2e372532b,
    0xca273eceea26619c,
    0xd186b8c721c0c207,
    0xeada7dd6cde0eb1e,
    0xf57d4f7fee6ed178,
    0x06f067aa72176fba,
    0x0a637dc5a2c898a6,
    0x113f9804bef90dae,
    0x1b710b35131c471b,
    0x28db77f523047d84,
    0x32caab7b40c72493,
    0x3c9ebe0a15c9bebc,
    0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6,
    0x597f299cfc657e2a,
    0x5fcb6fab3ad6faec,
    0x6c44198c4a475817,
];

// first 64 bits of the fractional part of the first 8 prime square root
const INITHASH: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];
