use deserr::Deserr;

#[derive(Deserr)]
#[deserr(error = deserr::Error)]
struct UnitStruct;

fn main() {}
