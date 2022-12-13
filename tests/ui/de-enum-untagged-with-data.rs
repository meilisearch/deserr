use deserr::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[deserr(error = deserr::Error)]
enum Enum {
    EmptyVariant,
    VariantWithSomething { data: usize },
}

fn main() {}
