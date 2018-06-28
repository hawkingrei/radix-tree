#[macro_export]
macro_rules! rep_no_copy {
    ($t:ty; $e:expr; $n:expr) => {{
        let mut v: Vec<$t> = Vec::with_capacity($n);
        for _ in 0..$n {
            v.push($e);
        }
        v
    }};
}

pub trait Digital {
    // TODO: consider providing a more efficient interface here (e.g. passing a slice directly)
    type I: Iterator<Item = u8>;
    const STOP_CHARACTER: Option<u8> = None;
    fn digits(&self) -> Self::I;
}
