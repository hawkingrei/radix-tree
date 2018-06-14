pub struct Node256<K, T>
where
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    children: Vec<ArtNode<K, T>>,
    marker: PhantomData<T>,
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

impl<K, V> Drop for Node256<K, V>
where
    V: 'static + Send + Sync,
{
    fn drop(&mut self) {
        for i in 0..256 {
            drop(&mut self.children[i as usize]);
        }
    }
}
