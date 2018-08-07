use internal::Digital;
use node::ArtNode::Empty;
use node::ArtNodeTrait;
use node::{ArtKey, ArtNode};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A simple lock-free radix tree.
pub struct Radix<K, V>
where
    K: Default + PartialEq + Digital + ArtKey,
    V: 'static + Send + Sync,
{
    head: ArtNode<K, V>,
    size: AtomicUsize,
    phantom: PhantomData<K>,
}

impl<K: ArtKey, T> Default for Radix<K, T>
where
    K: Default + PartialEq + Digital + ArtKey,
    T: 'static + Send + Sync,
{
    fn default() -> Self {
        Radix {
            head: ArtNode::Empty,
            size: AtomicUsize::new(0),
            phantom: Default::default(),
        }
    }
}

impl<K: ArtKey, T> Radix<K, T>
where
    K: Default + PartialEq + Digital + ArtKey,
    T: 'static + Send + Sync,
{
    fn new(level: usize) -> Self {
        Radix {
            head: ArtNode::Empty,
            size: AtomicUsize::new(0),
            phantom: Default::default(),
        }
    }

    fn insert_rec(
        parent: &mut ArtNode<K, T>,
        root: &mut ArtNode<K, T>,
        depth: usize,
        key: K,
        value: T,
    ) {
        match root {
            ArtNode::Empty => print!("1"),
            ArtNode::Inner4(ptr) => loop {
                ptr.header.read_lock_or_restart();
            },
            ArtNode::Inner16(ptr) => loop {
                ptr.header.read_lock_or_restart();
            },
            ArtNode::Inner48(ptr) => loop {
                ptr.header.read_lock_or_restart();
            },
            ArtNode::Inner256(ptr) => loop {
                ptr.header.read_lock_or_restart();
            },
            ArtNode::Value(ptr) => print!("1"),
        }
    }

    fn insert(&mut self, key: K, value: T) {
        Self::insert_rec(&mut ArtNode::Empty, &mut self.head, 0, key, value);
        self.size.fetch_add(1, Ordering::SeqCst);
    }
}
