use deserr::Deserr;

#[derive(Deserr)]
#[deserr(try_from(String) = String::parse -> usize)]
#[deserr(from(String) = usize::FromStr)]
struct UnitStruct {
    hello: usize,
}

fn main() {}
