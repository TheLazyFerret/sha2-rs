//! Author: TheLazyFerret (https://github.com/TheLazyFerret)
//! Copyright (c) 2025 TheLazyFerret
//! Licensed under the MIT License. See LICENSE file in the project root for full license information.
//!
//! Main source file

pub mod sha256;

use std::fs::File;
use std::env;
use crate::sha256::Sha256;

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() != 2 {
    eprintln!("incorrect number of parameters");
  }
  let file = File::open(args[1].clone()).expect("File not found");
  let mut x = Sha256::new();
  x.from_file(file);
  println!("{x} {} SHA256", args[1]);
}
