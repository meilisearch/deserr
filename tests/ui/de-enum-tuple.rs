use jayson::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[jayson(error = jayson::Error, tag = "t")]
enum Enum {
    Variant(i32),
}

fn main() {}
