use deserr::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[deserr(error = deserr::Error, tag = "t")]
enum Enum {
    Variant(i32),
}

fn main() {}
