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

pub trait ArtKey {
    fn bytes(&self) -> &[u8];
}

pub struct node_header {
    node_type: node_type,
    num_children: u8,
    partial: [u8; MAX_PREFIX_LEN],
    partial_len: usize,
}

pub struct Node<K, T>
where
    K: ArtKey + Send + 'static,
    T: 'static + Send + Sync,
{
    inner: Atomic<K>,
    header: node_header,
    keys: Vec<Atomic<Node<K, T>>>,
    children: Vec<Atomic<Node<K, T>>>,
}

/// A simple lock-free radix tree.
pub struct Radix<K, T>
where
    K: ArtKey + Send + 'static,
    T: 'static + Send + Sync,
{
    head: Atomic<Node<K, T>>,
    size: usize,
}

pub trait ArtNodeTrait<K, V>
where
    K: ArtKey + Send + 'static,
    V: 'static + Send + Sync,
{
    fn add_child(&mut self, node: Node<K, V>, byte: u8);

    fn clean_child(&mut self, byte: u8) -> bool;

    #[inline]
    fn is_full(&self) -> bool;

    fn grow_and_add(self, leaf: Node<K, V>, byte: u8) -> Node<K, V>;

    fn shrink(self) -> Node<K, V>;

    #[inline]
    fn mut_header(&mut self) -> &mut node_header;

    #[inline]
    fn header(&self) -> &node_header;

    #[inline]
    fn find_child_mut(&mut self, byte: u8) -> &mut Node<K, V>;

    #[inline]
    fn find_child(&self, byte: u8) -> Option<&Node<K, V>>;

    #[inline]
    fn has_child(&self, byte: u8) -> bool;

    #[inline]
    fn to_art_node(self: Box<Self>) -> Node<K, V>;
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
