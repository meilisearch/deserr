use jayson::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[jayson(error = jayson::Error)]
struct TupleStruct(i32, i32);

fn main() {}
