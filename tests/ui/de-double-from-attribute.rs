use deserr::Deserr;

#[derive(Deserr)]
struct UnitStruct {
    #[deserr(from(String) = String::parse -> usize)]
    #[deserr(from(String) = usize::FromStr -> usize)]
    hello: usize,
}

fn main() {}
