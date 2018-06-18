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
