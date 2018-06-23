extern crate crossbeam_epoch;
extern crate crossbeam_utils;
#[cfg(feature = "use_std")]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate memoffset;
#[macro_use]
extern crate scopeguard;

#[macro_use]
mod internal;
mod epoch;
mod node;
mod node16;
mod node256;
mod node4;
mod node48;
mod tree;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
