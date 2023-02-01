use deserr::Deserr;

#[derive(Deserr)]
struct UnitStruct {
    #[deserr(from(String) = usize::FromStr, try_from(String) = String::parse -> usize)]
    hello: usize,
}

fn main() {}
