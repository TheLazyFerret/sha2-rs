// Author: TheLazyFerret (https://github.com/TheLazyFerret)
// Copyright (c) 2025 TheLazyFerret
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

// size (in bits) of a u64
const U64SIZE: usize = size_of::<u64>() * 8;

// size (in bits) of a u32
const U32SIZE: usize = size_of::<u32>() * 8;

// The right shift. As a function for easier syntax

pub trait BasicFunctions {
    fn right_shift(&self, n: usize) -> Self;
    fn right_rotation(&self, n: usize) -> Self;
}

impl BasicFunctions for u64 {
    fn right_rotation(&self, n: usize) -> Self {
        assert!(n < U64SIZE);
        (self >> n) | (self << (U64SIZE - n))
    }

    fn right_shift(&self, n: usize) -> Self {
        assert!(n < U64SIZE);
        self >> n
    }
}

impl BasicFunctions for u32 {
    fn right_rotation(&self, n: usize) -> Self {
        assert!(n < U32SIZE);
        (self >> n) | (self << (U32SIZE - n))
    }

    fn right_shift(&self, n: usize) -> Self {
        assert!(n < U32SIZE);
        self >> n
    }
}
