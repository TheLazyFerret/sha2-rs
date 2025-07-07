// Author: TheLazyFerret (https://github.com/TheLazyFerret)
// Copyright (c) 2025 TheLazyFerret
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

mod basic_functions;
mod sha256;
mod sha512;

use std::fmt::Write;
use std::fs;

pub struct Sha512 {
    hash: [u64; 8],
}

impl Sha512 {
    pub fn print(&self) {
        for n in self.hash {
            print!("{:0>8x}", n);
        }
        println!();
    }
    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for n in self.hash {
            write!(&mut string, "{:0>8x}", n)
                .expect("error reading from hash array. May be empty?");
        }
        return string;
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        let mut array: [u8; 64] = [8; 64];
        let mut iteration: usize = 0;
        for n in self.hash {
            let splited: [u8; 8] = split_u64_u8(n);
            array[(iteration * 8)..(iteration * 8 + 8)].copy_from_slice(splited.as_slice());
            iteration += 1;
        }
        return array;
    }
} // impl Sha512

pub struct Sha256 {
    hash: [u32; 8],
}

impl Sha256 {
    pub fn print(&self) {
        for n in self.hash {
            print!("{:0>4x}", n);
        }
        println!();
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for n in self.hash {
            write!(&mut string, "{:0>4x}", n)
                .expect("error reading from hash array. May be empty?");
        }
        return string;
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut array: [u8; 32] = [8; 32];
        let mut iteration: usize = 0;
        for n in self.hash {
            let splited: [u8; 4] = split_u32_u8(n);
            array[(iteration * 4)..(iteration * 4 + 4)].copy_from_slice(splited.as_slice());
            iteration += 1;
        }
        return array;
    }
}

pub trait Hash {
    fn from_file(file: &fs::File) -> Self;
    fn from_string(string: &str) -> Self;
}

impl Hash for Sha512 {
    fn from_file(file: &fs::File) -> Self {
        return sha512::hash_from_file(file);
    }
    fn from_string(string: &str) -> Self {
        return sha512::hash_from_string(string);
    }
}

impl Hash for Sha256 {
    fn from_file(file: &fs::File) -> Self {
        return sha256::hash_from_file(file);
    }
    fn from_string(string: &str) -> Self {
        return sha256::hash_from_string(string);
    }
}

// Return an array in BIG ENDIAN
pub fn split_u64_u8(number: u64) -> [u8; 8] {
    let mut array: [u8; 8] = [0; 8];
    for n in 0..8 {
        array[n] = (number >> (7 - n) * 8) as u8;
    }
    return array;
}

pub fn split_u32_u8(number: u32) -> [u8; 4] {
    let mut array: [u8; 4] = [0; 4];
    for n in 0..4 {
        array[n] = (number >> (3 - n) * 8) as u8;
    }
    return array;
}

#[cfg(test)]
mod tests {
    use crate::Sha256;
    use crate::Sha512;
    use std::fmt::Write;

    #[test]
    fn sha512_parity() {
        let random_text = String::from("a very random text");
        let file = std::fs::File::open("./foo.txt").expect("error opeing file");
        //file.write(random_text.as_bytes()).expect("error writting text");

        let x: Sha512 = crate::Hash::from_string(&random_text);
        let y: Sha512 = crate::Hash::from_file(&file);
        assert_eq!(x.hash, y.hash);
    }

    #[test]
    fn sha512_methods() {
        let random_text = String::from("something very not random");
        let hash: Sha512 = crate::Hash::from_string(&random_text);
        let x = hash.to_string();
        let mut y: String = String::new();

        let bytes = hash.to_bytes();
        for n in bytes {
            write!(&mut y, "{:0>2x}", n).unwrap();
        }
        assert_eq!(x, y);
    }

    #[test]
    fn sha256_parity() {
        let random_text = String::from("a very random text");
        let file = std::fs::File::open("./foo.txt").expect("error opeing file");
        //file.write(random_text.as_bytes()).expect("error writting text");

        let x: Sha256 = crate::Hash::from_string(&random_text);
        let y: Sha256 = crate::Hash::from_file(&file);
        assert_eq!(x.hash, y.hash);
    }

    #[test]
    fn sha256_methods() {
        let random_text = String::from("something very not random");
        let hash: Sha256 = crate::Hash::from_string(&random_text);
        let x = hash.to_string();
        let mut y: String = String::new();

        let bytes = hash.to_bytes();
        for n in bytes {
            write!(&mut y, "{:x}", n).unwrap();
        }
        assert_eq!(x, y);
    }
}
