use internal::Digital;
use node::ArtNode::Empty;
use node::ArtNodeTrait;
use node::{ArtKey, ArtNode};
use std::marker::PhantomData;

/// A simple lock-free radix tree.
pub struct Radix<K, V>
where
    K: Default + PartialEq + Digital + ArtKey,
    V: 'static + Send + Sync,
{
    head: Box<ArtNode<K, V>>,
    size: usize,
    phantom: PhantomData<K>,
}

impl<K: ArtKey, T> Default for Radix<K, T>
where
    K: Default + PartialEq + Digital + ArtKey,
    T: 'static + Send + Sync,
{
    fn default() -> Self {
        Radix {
            head: Box::new(ArtNode::Empty),
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
            head: Box::new(ArtNode::Empty),
            size: 0,
            phantom: Default::default(),
        }
    }

    fn insert(&mut self, key: u64, value: T) {
        let mut parentKey: u8 = 0;
        let mut nodeKey: u8 = 0;
        if matches!(self.head, ArtNode::Empty) {
            self.head = Box::new(ArtNode::Inner4(Box::new(ArtNodeTrait::new())));
        }
        let mut node: Box<ArtNode<K, T>> = Box::new(ArtNode::Empty);
        //let mut nextNode: Box<ArtNode<K, T>> = self.head;

        loop {
            //node = nextNode
        }
    }
}
