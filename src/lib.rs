extern crate crossbeam_epoch;
extern crate crossbeam_utils;

use crossbeam_epoch::{pin, unprotected, Atomic, Guard, Owned, Shared};
use std::marker::PhantomData;

const RADIX_TREE_MAP_SHIFT: usize = 6;
const MAX_PREFIX_LEN: usize = 6;

enum node_type {
    NODE4,
    NODE16,
    NODE48,
    NODE256,
}

pub enum ArtNode<K, V>
where
    V: 'static + Send + Sync,
{
    Empty,

    Inner4(Box<Node4<K, V>>),
    Inner16(Box<Node16<K, V>>),
    Inner48(Box<Node48<K, V>>),
    Inner256(Box<Node256<K, V>>),
    //LeafLarge(Box<(K, V)>),
    //LeafLargeKey(Box<K>, SmallStruct<V>),
    //LeafLargeValue(SmallStruct<K>, Box<V>),
    //LeafSmall(SmallStruct<K>, SmallStruct<V>),
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

pub struct Node4<K, T>
where
    T: 'static + Send + Sync,
{
    header: node_header,
    keys: [Atomic<K>; 4],
    children: [Atomic<ArtNode<K, T>>; 4],
}

pub struct Node16<K, T>
where
    T: 'static + Send + Sync,
{
    header: node_header,
    keys: [Atomic<K>; 16],
    children: [Atomic<ArtNode<K, T>>; 16],
}

pub struct Node48<K, T>
where
    T: 'static + Send + Sync,
{
    header: node_header,
    keys: [Atomic<K>; 256],
    children: [Atomic<ArtNode<K, T>>; 48],
}

pub struct Node256<K, T>
where
    T: 'static + Send + Sync,
{
    header: node_header,
    children: [Atomic<ArtNode<K, T>>; 48],
}

/// A simple lock-free radix tree.
pub struct Radix<'a, K: 'a, T: 'a>
where
    T: 'static + Send + Sync,
{
    head: Atomic<ArtNode<K, T>>,
    size: usize,
    phantom: PhantomData<&'a K>,
}

pub trait ArtNodeTrait<K, V>
where
    V: 'static + Send + Sync,
{
    fn add_child(&mut self, node: ArtNode<K, V>, byte: u8);

    fn clean_child(&mut self, byte: u8) -> bool;

    #[inline]
    fn is_full(&self) -> bool;

    fn grow_and_add(self, leaf: ArtNode<K, V>, byte: u8) -> ArtNode<K, V>;

    fn shrink(self) -> ArtNode<K, V>;

    #[inline]
    fn mut_header(&mut self) -> &mut node_header;

    #[inline]
    fn header(&self) -> &node_header;

    #[inline]
    fn find_child_mut(&mut self, byte: u8) -> &mut ArtNode<K, V>;

    #[inline]
    fn find_child(&self, byte: u8) -> Option<&ArtNode<K, V>>;

    #[inline]
    fn has_child(&self, byte: u8) -> bool;

    #[inline]
    fn to_art_node(self: Box<Self>) -> ArtNode<K, V>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
