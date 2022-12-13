use deserr::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[deserr(error = deserr::Error)]
enum Enum {
    EmptyVariant,
    VariantWithSomething(u16),
}

fn main() {}
