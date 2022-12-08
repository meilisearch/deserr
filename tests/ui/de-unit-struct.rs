use deserr::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[deserr(error = deserr::Error)]
struct UnitStruct;

fn main() {}
