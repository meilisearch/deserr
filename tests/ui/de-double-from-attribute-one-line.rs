use deserr::DeserializeFromValue;

#[derive(DeserializeFromValue)]
struct UnitStruct {
    #[deserr(from(String) = String::parse -> usize, from(String) = usize::FromStr -> usize)]
    hello: usize,
}

fn main() {}
