use deserr::Deserr;

#[derive(Deserr)]
#[deserr(error = deserr::Error)]
enum Enum {
    EmptyVariant,
    VariantWithSomething { data: usize },
}

fn main() {}
