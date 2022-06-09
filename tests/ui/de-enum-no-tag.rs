use jayson::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[jayson(error = jayson::Error)]
enum Enum {
    Variant,
}

fn main() {}
