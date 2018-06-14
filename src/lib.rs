#![feature(use_extern_macros)]
#![feature(fixed_size_array)]
extern crate crossbeam_epoch;
extern crate crossbeam_utils;

#[macro_use]
mod node;
mod node16;
mod node256;
mod node4;
mod node48;
mod tree;

use crossbeam_epoch::{pin, unprotected, Atomic, Guard, Owned, Shared};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const RADIX_TREE_MAP_SHIFT: usize = 6;
const EMPTY_CELL: u8 = 0;

pub const SMALL_STRUCT: usize = 8;
type Small = [u8; SMALL_STRUCT];
