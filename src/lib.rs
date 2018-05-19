extern crate crossbeam_epoch;
extern crate crossbeam_utils;

use crossbeam_epoch::{pin, unprotected, Atomic, Guard, Owned, Shared};

const RADIX_TREE_MAP_SHIFT: usize = 6;
const MAX_PREFIX_LEN: usize = 6;

enum node_type {
    NODE4,
    NODE16,
    NODE48,
    NODE256,
}

struct node_header {
    node_type: node_type,
    num_children: u8,
    partial: [u8; MAX_PREFIX_LEN],
    partial_len: usize,
}

struct Node<T: Send + 'static> {
    inner: Atomic<T>,
    keys: Vec<Atomic<Node<T>>>,
    children: Vec<Atomic<Node<T>>>,
}

/// A simple lock-free radix tree.
pub struct Radix<T>
where
    T: 'static + Send + Sync,
{
    head: Atomic<Node<T>>,
}

//pub struct Node4<'a> {
//    ntype: u8,
//    key: [u8; 4],
//    value: [&'a Node; 4],
//}

//pub struct Node16<'a> {
//    ntype: u8,
//    key: [u8; 16],
//    value: [&'a Node; 16],
//}

//pub struct Node48<'a> {
//    ntype: u8,
//    key: [u8; 256],
//    value: [&'a Node; 48],
//}

//pub struct Node256<'a> {
//    ntype: u8,
//    key: [u8; 256],
//    value: [&'a Node; 48],
//}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
