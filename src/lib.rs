#![feature(fixed_size_array)]
extern crate crossbeam_epoch;
extern crate crossbeam_utils;

use crossbeam_epoch::{pin, unprotected, Atomic, Guard, Owned, Shared};
use std::marker::PhantomData;
use std::{mem, ptr};

const RADIX_TREE_MAP_SHIFT: usize = 6;
const MAX_PREFIX_LEN: usize = 6;
const EMPTY_CELL: u8 = 0;

pub const SMALL_STRUCT: usize = 8;
type Small = [u8; SMALL_STRUCT];

macro_rules! rep_no_copy {
    ($t:ty ;$e:expr; $n:expr) => {{
        let mut v:[$t;$n]  = unsafe { mem::uninitialized() };
        for i in 0..$n {
            v[i] = $e;
        }
        v
    }};
}

pub struct SmallStruct<T> {
    storage: Small,
    marker: PhantomData<T>,
}

impl<T> SmallStruct<T> {
    pub fn new(elem: T) -> Self {
        unsafe {
            let mut ret = SmallStruct {
                storage: mem::uninitialized(),
                marker: PhantomData,
            };
            std::ptr::copy_nonoverlapping(
                &elem as *const T as *const u8,
                ret.storage.as_mut_ptr(),
                mem::size_of::<T>(),
            );
            ret
        }
    }

    pub fn reference(&self) -> &T {
        unsafe { &*(self.storage.as_ptr() as *const T) }
    }

    pub fn own(self) -> T {
        unsafe {
            let mut ret = mem::uninitialized();
            let dst = &mut ret as *mut T as *mut u8;
            std::ptr::copy_nonoverlapping(self.storage.as_ptr(), dst, mem::size_of::<T>());
            ret
        }
    }
}

enum NodeType {
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

pub struct NodeHeader {
    //NodeType: NodeType,
    version: Atomic<u64>,
    num_children: u8,
    partial: [u8; MAX_PREFIX_LEN],
    partial_len: usize,
}

impl NodeHeader {
    pub fn new() -> Self {
        NodeHeader {
            version: Atomic::new(0),
            num_children: 0,
            partial_len: 0,
            partial: unsafe { mem::uninitialized() },
        }
    }

    pub fn compute_prefix_match<K: ArtKey>(&self, key: &K, depth: usize) -> usize {
        for i in 0..self.partial_len {
            if key.bytes()[i + depth] != self.partial[i] {
                return i;
            }
        }
        self.partial_len
    }
}

pub struct Node4<K, T>
where
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    keys: [K; 4],
    children: [ArtNode<K, T>; 4],
    marker: PhantomData<T>,
}

pub struct Node16<K, T>
where
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    keys: [K; 16],
    children: [ArtNode<K, T>; 16],
    marker: PhantomData<T>,
}

pub struct Node48<K, T>
where
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    keys: [K; 256],
    children: [ArtNode<K, T>; 48],
    marker: PhantomData<T>,
}

pub struct Node256<K, T>
where
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    children: [ArtNode<K, T>; 256],
    marker: PhantomData<T>,
}

/// A simple lock-free radix tree.
pub struct Radix<'a, K: 'a + ArtKey, V: 'a>
where
    V: 'static + Send + Sync,
{
    head: ArtNode<K, V>,
    size: usize,
    level: usize,
    phantom: PhantomData<&'a K>,
}

impl<'a, K: ArtKey, T> Default for Radix<'a, K, T>
where
    K: 'a + ArtKey,
    T: 'static + Send + Sync,
{
    fn default() -> Self {
        Radix {
            head: ArtNode::Empty,
            level: 0,
            size: 0,
            phantom: Default::default(),
        }
    }
}

impl<'a, K: ArtKey, T> Radix<'a, K, T>
where
    K: 'a + ArtKey,
    T: 'static + Send + Sync,
{
    fn new(level: usize) -> Self {
        Radix {
            head: ArtNode::Empty,
            level: level,
            size: 0,
            phantom: Default::default(),
        }
    }
}

trait ArtNodeTrait<K, V>
where
    V: 'static + Send + Sync,
{
    fn new() -> Self;

    //fn add_child(&mut self, node: ArtNode<K, V>, byte: u8);

    //fn clean_child(&mut self, byte: u8) -> bool;

    //#[inline]
    //fn is_full(&self) -> bool;

    //fn grow_and_add(self, leaf: ArtNode<K, V>, byte: u8) -> ArtNode<K, V>;

    //fn shrink(self) -> ArtNode<K, V>;

    //#[inline]
    //fn mut_header(&mut self) -> &mut NodeHeader;

    //#[inline]
    //fn header(&self) -> &NodeHeader;

    //#[inline]
    //fn find_child_mut(&mut self, byte: u8) -> &mut ArtNode<K, V>;

    //#[inline]
    //fn find_child(&self, byte: u8) -> Option<&ArtNode<K, V>>;

    //#[inline]
    //fn has_child(&self, byte: u8) -> bool;

    //#[inline]
    //fn to_art_node(self: Box<Self>) -> ArtNode<K, V>;
}

impl<K, V> ArtNodeTrait<K, V> for Node4<K, V>
where
    K: Default,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node4 {
            header: NodeHeader::new(),
            keys: rep_no_copy!(K; Default::default(); 4),
            children: rep_no_copy!(ArtNode<K, V>; ArtNode::Empty; 4),
            marker: Default::default(),
        }
    }
}

impl<K, V> Drop for Node4<K, V>
where
    V: 'static + Send + Sync,
{
    fn drop(&mut self) {
        for i in 0..self.header.num_children {
            drop(&mut self.children[i as usize]);
        }
    }
}

impl<K, V> ArtNodeTrait<K, V> for Node16<K, V>
where
    K: Default,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node16 {
            header: NodeHeader::new(),
            keys: rep_no_copy!(K; Default::default(); 16),
            children: rep_no_copy!(ArtNode<K, V>; ArtNode::Empty; 16),
            marker: Default::default(),
        }
    }
}

impl<K, V> Drop for Node16<K, V>
where
    V: 'static + Send + Sync,
{
    fn drop(&mut self) {
        for i in 0..self.header.num_children {
            drop(&mut self.children[i as usize]);
        }
    }
}

impl<K, V> ArtNodeTrait<K, V> for Node48<K, V>
where
    K: Default,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node48 {
            header: NodeHeader::new(),
            keys: rep_no_copy!(K; Default::default(); 256),
            children: rep_no_copy!(ArtNode<K, V>; ArtNode::Empty; 48),
            marker: Default::default(),
        }
    }
}

impl<K, V> Drop for Node48<K, V>
where
    V: 'static + Send + Sync,
{
    fn drop(&mut self) {
        for i in 0..self.header.num_children {
            drop(&mut self.children[i as usize]);
        }
    }
}

impl<K, V> ArtNodeTrait<K, V> for Node256<K, V>
where
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node256 {
            header: NodeHeader::new(),
            children: rep_no_copy!(ArtNode<K, V>; ArtNode::Empty;  256),
            marker: Default::default(),
        }
    }
}
