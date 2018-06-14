use node::{ArtKey, ArtNode};
use std::marker::PhantomData;

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
