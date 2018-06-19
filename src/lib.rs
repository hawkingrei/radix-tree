#![feature(use_extern_macros)]
#![feature(fixed_size_array)]
extern crate crossbeam_epoch;
extern crate crossbeam_utils;

#[macro_use]
mod internal;
mod epoch;
mod node;
mod node16;
mod node256;
mod node4;
mod node48;
mod tree;

use crossbeam_epoch::{pin, unprotected, Atomic, Guard, Owned, Shared};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
