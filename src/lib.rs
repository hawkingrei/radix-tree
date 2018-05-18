const RADIX_TREE_MAP_SHIFT: usize = 6;

const NODE4: u8 = 1;
const NODE16: u8 = 2;
const NODE48: u8 = 3;
const NODE256: u8 = 4;

pub trait Node {}

pub struct Node4<'a> {
    key: [u8; 4],
    value: [&'a Node; 4],
}

pub struct Node16<'a> {
    key: [u8; 16],
    value: [&'a Node; 16],
}

pub struct Node48<'a> {
    key: [u8; 256],
    value: [&'a Node; 48],
}

pub struct Node256<'a> {
    key: [u8; 256],
    value: [&'a Node; 48],
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
