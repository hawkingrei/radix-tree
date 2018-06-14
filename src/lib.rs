#![feature(fixed_size_array)]
extern crate crossbeam_epoch;
extern crate crossbeam_utils;

use crossbeam_epoch::{pin, unprotected, Atomic, Guard, Owned, Shared};
use std::arch::x86_64::_mm_pause;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::{mem, ptr};

const RADIX_TREE_MAP_SHIFT: usize = 6;
const MAX_PREFIX_LEN: usize = 6;
const EMPTY_CELL: u8 = 0;

pub const SMALL_STRUCT: usize = 8;
type Small = [u8; SMALL_STRUCT];

macro_rules! rep_no_copy {
    ($t:ty; $e:expr; $n:expr) => {{
        let mut v: Vec<$t> = Vec::with_capacity($n);
        for i in 0..$n {
            v.push($e);
        }
        v
    }};
}
