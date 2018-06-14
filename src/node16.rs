use node;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use std::marker::PhantomData;

pub struct Node16<K, T>
where
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    keys: Vec<K>,
    children: Vec<ArtNode<K, T>>,
    marker: PhantomData<T>,
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