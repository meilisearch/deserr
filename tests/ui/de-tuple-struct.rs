use deserr::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[deserr(error = deserr::Error)]
struct TupleStruct(i32, i32);

fn main() {}
