use jayson::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[jayson(error = jayson::Error)]
struct UnitStruct;

fn main() {}
