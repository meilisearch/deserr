use deserr::Deserr;

#[derive(Deserr)]
struct UnitStruct {
    #[deserr(try_from(String) = String::parse -> usize)]
    #[deserr(try_from(String) = usize::FromStr -> usize)]
    hello: usize,
}

fn main() {}
