use deserr::Deserr;

#[derive(Deserr)]
#[deserr(error = deserr::Error)]
enum Enum {
    EmptyVariant,
    VariantWithSomething(u16),
}

fn main() {}
