use internal::Digital;
use node;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use std::cmp::PartialEq;
use std::marker::PhantomData;

pub struct Node48<K, T>
where
    K: Default + PartialEq + Digital,
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    keys: Vec<u8>,
    children: Vec<ArtNode<K, T>>,
    marker: PhantomData<T>,
}

impl<K, V> ArtNodeTrait<K, V> for Node48<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node48 {
            header: NodeHeader::new(),
            keys: rep_no_copy!(u8; 0; 256),
            children: rep_no_copy!(ArtNode<K, V>; ArtNode::Empty; 48),
            marker: Default::default(),
        }
    }

    fn add_child(&mut self, node: ArtNode<K, V>, byte: u8) {}

    fn is_full(&self) -> bool {
        self.header.num_children >= 48
    }

    #[inline]
    fn find_child_mut(
        &mut self,
        byte: usize,
        level: usize,
        parent: ArtNode<K, V>,
        version_parent: usize,
    ) -> Result<&mut ArtNode<K, V>, bool> {
        return Err(false);
    }
}

impl<K, V> Drop for Node48<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn drop(&mut self) {
        for i in 0..256 {
            drop(&mut self.children[i as usize]);
        }
    }
}
