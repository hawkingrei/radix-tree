pub struct Node48<K, T>
where
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    keys: Vec<K>,
    children: Vec<ArtNode<K, T>>,
    marker: PhantomData<T>,
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
        for i in 0..256 {
            drop(&mut self.children[i as usize]);
        }
    }
}
