//! Author: TheLazyFerret (https://github.com/TheLazyFerret)
//! Copyright (c) 2025 TheLazyFerret
//! Licensed under the MIT License. See LICENSE file in the project root for full license information.
//!
//! Main source file

pub mod sha256;

use std::fs::File;

use crate::sha256::Sha256;

fn main() {
  let file = File::open("../kubuntu-25.04-desktop-amd64.iso").expect("error opening");
  let mut x = Sha256::new();
  x.from_file(file);
  println!("{x}");
}
