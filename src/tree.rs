use internal::Digital;
use node::{ArtKey, ArtNode};
use std::marker::PhantomData;

/// A simple lock-free radix tree.
pub struct Radix<K, V>
where
    K: Default + PartialEq + Digital + ArtKey,
    V: 'static + Send + Sync,
{
    head: ArtNode<K, V>,
    size: usize,
    level: usize,
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
            level: 0,
            size: 0,
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
            level: level,
            size: 0,
            phantom: Default::default(),
        }
    }
}
