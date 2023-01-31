use deserr::Deserr;

#[derive(Deserr)]
#[deserr(error = deserr::Error)]
union Union {
    x: i32,
}

fn main() {}
