// Author: TheLazyFerret (https://github.com/TheLazyFerret)
// Copyright (c) 2025 TheLazyFerret
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use crate::basic_functions::BasicFunctions;
use std::fs;
use std::io;

struct IterationBlock {
    bytes_readed: u64,
    bit_padded: bool,
    finished: bool,
    buffer: [u8; 64],
}

impl IterationBlock {
    fn new() -> Self {
        let iblock = IterationBlock {
            bytes_readed: 0,
            bit_padded: false,
            finished: false,
            buffer: [0; 64],
        };
        return iblock;
    }

    fn parse_block(&self) -> [u32; 16] {
        let mut x: [u32; 16] = [0; 16];
        let mut iteration: usize = 0;
        while iteration < 16 {
            x[iteration] = fuse_32(&self.buffer[(iteration * 4)..(iteration * 4 + 4)]);
            iteration += 1;
        }
        return x;
    }
}

fn fuse_32(array: &[u8]) -> u32 {
    assert_eq!(array.len(), 4);
    let mut x: u32 = 0;
    let mut iteration = 0;
    for n in array {
        let mut aux: u32 = *n as u32;
        aux = aux << (24 - (iteration * 8));
        x = x | aux;
        iteration += 1;
    }
    return x;
}

// fragment a u64 value into an array [u8; 8] BIG ENDIAN
fn fragment_u64(number: u64) -> [u8; 8] {
    let mut help_bytes: [u8; 8] = [0; 8];
    let mut iteration: i32 = 7;
    while iteration >= 0 {
        help_bytes[iteration as usize] = (number >> (iteration * 8)) as u8;
        iteration -= 1;
    }
    return help_bytes;
}

// function to get the next block to computer froma buffer
fn get_next_block<T: std::io::Read>(
    file_buffer: &mut io::BufReader<T>,
    iterator: &mut IterationBlock,
) {
    iterator.buffer = [0; 64];
    let n_bytes_readed: usize =
        io::Read::read(file_buffer, &mut iterator.buffer).expect("error reading buffer");
    iterator.bytes_readed += n_bytes_readed as u64;

    // last block readed
    if n_bytes_readed < 56 {
        if iterator.bit_padded == false {
            iterator.buffer[n_bytes_readed] = 0x80;
            iterator.bit_padded = true;
        }
        let size: [u8; 8] = fragment_u64(iterator.bytes_readed * 8);
        let mut iter: i8 = 7;
        while iter >= 0 {
            iterator.buffer[(56 + iter) as usize] = size[(7 - iter) as usize];
            iter -= 1;
        }
        iterator.finished = true;
    }
    // not the last block append
    else if n_bytes_readed >= 55 && n_bytes_readed < 64 {
        if iterator.bit_padded == false {
            iterator.buffer[n_bytes_readed] = 0x80;
            iterator.bit_padded = true;
        }
    }
} // fn get_next_block

// main function to calculate the hash of a file
pub fn hash_from_file(file: &fs::File) -> crate::Sha256 {
    let mut buffer_read = io::BufReader::new(file);
    let mut iterator = IterationBlock::new();
    let mut hash: [u32; 8] = INITHASH;
    while iterator.finished == false {
        get_next_block(&mut buffer_read, &mut iterator);
        let parsed_block: [u32; 16] = iterator.parse_block();
        hash = compute_block(&parsed_block, &hash);
    }
    return crate::Sha256 { hash: hash };
}

// main function to calculate the hash of a string
pub fn hash_from_string(string: &str) -> crate::Sha256 {
    let mut buffer_read = io::BufReader::new(io::Cursor::new(string.to_string()));
    let mut iterator = IterationBlock::new();
    let mut hash: [u32; 8] = INITHASH;
    while iterator.finished == false {
        get_next_block(&mut buffer_read, &mut iterator);
        let parsed_block: [u32; 16] = iterator.parse_block();
        hash = compute_block(&parsed_block, &hash);
    }
    return crate::Sha256 { hash: hash };
}

// Compute the block, returning the updated hash value in each iteration
fn compute_block(block: &[u32; 16], hash: &[u32; 8]) -> [u32; 8] {
    let mut message: [u32; 64] = [0; 64];
    let mut aux_hash: [u32; 8] = *hash;
    // MESSAGE SCHEDULE PREPARATION
    message[0..16].clone_from_slice(&block[0..16]);
    for n in 16..64 {
        message[n] = calculate_schedule(&message, n);
    }
    // COMPUTE WORKING VARIABLS
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
    // COMPUTE INTERMEDIAL HASH
    let mut new_hash: [u32; 8] = [0; 8];
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

// Calculate the values of schedules for 16 <= n <= 79
fn calculate_schedule(message: &[u32; 64], iteration: usize) -> u32 {
    assert!(iteration > 15 && iteration < 64);
    let x: u32 = {
        lower_sigma1(message[iteration - 2])
            .wrapping_add(message[iteration - 7])
            .wrapping_add(lower_sigma0(message[iteration - 15]))
            .wrapping_add(message[iteration - 16])
    };
    return x;
}

// Calculate the vale of T1
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

// Calculate the vale of T2
fn calculate_t2(hash: &[u32; 8], iteration: usize) -> u32 {
    assert!(iteration < 64);
    let x: u32 = { upper_sigma0(hash[0]).wrapping_add(majority(hash[0], hash[1], hash[2])) };
    return x;
}

// BASIC FUNCTIONS
fn choose(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

fn majority(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn upper_sigma0(x: u32) -> u32 {
    BasicFunctions::right_rotation(&x, 2)
        ^ BasicFunctions::right_rotation(&x, 13)
        ^ BasicFunctions::right_rotation(&x, 22)
}

fn upper_sigma1(x: u32) -> u32 {
    BasicFunctions::right_rotation(&x, 6)
        ^ BasicFunctions::right_rotation(&x, 11)
        ^ BasicFunctions::right_rotation(&x, 25)
}

fn lower_sigma0(x: u32) -> u32 {
    BasicFunctions::right_rotation(&x, 7)
        ^ BasicFunctions::right_rotation(&x, 18)
        ^ BasicFunctions::right_shift(&x, 3)
}

fn lower_sigma1(x: u32) -> u32 {
    BasicFunctions::right_rotation(&x, 17)
        ^ BasicFunctions::right_rotation(&x, 19)
        ^ BasicFunctions::right_shift(&x, 10)
}

// CONSTANTS
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
