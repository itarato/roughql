use roughql_lib::{execute, parse};
use roughql_macro::GraphQLSource;
use std::rc::Rc;

#[derive(Default, GraphQLSource)]
struct Bar {
    #[graphql_field(kind = "int")]
    value: i64,
}

#[derive(Default, GraphQLSource)]
struct Foo {
    #[graphql_field(kind = "int")]
    value: i64,
    #[graphql_field(kind = "obj")]
    bar: Rc<Bar>,
}

#[derive(Default, GraphQLSource)]
struct Root {
    #[graphql_field(kind = "obj")]
    foo: Rc<Foo>,
}

fn main() {
    let input = "query {
        foo {
            value
            bar {
                value
            }
        }
    }";

    let query = parse(input).unwrap();
    dbg!(&query);

    let out = execute(query, Rc::new(Root::default()));
    println!("{}", out);
}
