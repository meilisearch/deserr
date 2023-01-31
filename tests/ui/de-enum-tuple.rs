use deserr::Deserr;

#[derive(Deserr)]
#[deserr(error = deserr::Error, tag = "t")]
enum Enum {
    Variant(i32),
}

fn main() {}
