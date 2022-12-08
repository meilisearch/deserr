use deserr::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[deserr(error = deserr::Error)]
enum Enum {
    Variant,
}

fn main() {}
