use internal::Digital;
use node;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use std::cmp::PartialEq;
use std::marker::PhantomData;

pub struct Node256<K, T>
where
    K: Default + PartialEq + for<'a> Digital<'a>,
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    children: Vec<ArtNode<K, T>>,
    marker: PhantomData<T>,
}

impl<K, V> ArtNodeTrait<K, V> for Node256<K, V>
where
    K: Default + PartialEq + for<'a> Digital<'a>,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node256 {
            header: NodeHeader::new(),
            children: rep_no_copy!(ArtNode<K, V>; ArtNode::Empty;  256),
            marker: Default::default(),
        }
    }

    fn add_child(&mut self, node: ArtNode<K, V>, byte: u8) {}

    fn is_full(&self) -> bool {
        self.header.num_children >= 256
    }

    #[inline]
    fn find_child_mut(
        &mut self,
        byte: usize,
        level: usize,
        parent: ArtNode<K, V>,
        version_parent: usize,
    ) -> &mut Result<&mut ArtNode<K, V>, bool> {
        return &mut Err(false);
    }
}

impl<K, V> Drop for Node256<K, V>
where
    K: Default + PartialEq + for<'a> Digital<'a>,
    V: 'static + Send + Sync,
{
    fn drop(&mut self) {
        for i in 0..256 {
            drop(&mut self.children[i as usize]);
        }
    }
}
