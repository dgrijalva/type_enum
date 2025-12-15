use type_enum::TypeEnum;

#[derive(TypeEnum)]
enum DuplicateTypes {
    Foo(String),
    Bar(usize),
    Baz(String),
}

fn main() {}
