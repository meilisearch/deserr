use deserr::Deserr;

#[derive(Deserr)]
struct UnitStruct {
    #[deserr(try_from(String) = String::parse -> usize)]
    #[deserr(from(String) = usize::FromStr)]
    hello: usize,
}

fn main() {}
