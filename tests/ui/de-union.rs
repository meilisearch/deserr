use jayson::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[jayson(error = jayson::Error)]
union Union {
    x: i32,
}

fn main() {}
