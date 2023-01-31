use deserr::Deserr;

#[derive(Deserr)]
#[deserr(error = deserr::Error)]
struct TupleStruct(i32, i32);

fn main() {}
