use deserr::Deserr;

#[derive(Deserr)]
#[deserr(from(String) = usize::FromStr, try_from(String) = String::parse -> usize)]
struct UnitStruct {
    hello: usize,
}

fn main() {}
