use num_traits::Num;

pub fn plural(n: impl Num) -> &'static str {
    if n.is_one() {
        ""
    } else {
        "s"
    }
}
