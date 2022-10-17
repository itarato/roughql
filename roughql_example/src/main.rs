use roughql_lib::Query;
use roughql_lib::SchemaProvider;
use roughql_macro::GraphNode;
use std::rc::Rc;

#[derive(GraphNode)]
struct Bar {
    #[graphql_field(kind = "int")]
    value: i64,
}

impl Bar {
    fn new() -> Self {
        Self { value: 12 }
    }
}

#[derive(GraphNode)]
struct Foo {
    #[graphql_field(kind = "int")]
    value: i64,
    #[graphql_field(kind = "obj")]
    bar: Rc<Bar>,
    #[graphql_field(kind = "str")]
    name: String,
}

impl Foo {
    fn new() -> Self {
        Self {
            value: -43,
            bar: Rc::new(Bar::new()),
            name: "Steve".to_owned(),
        }
    }
}

#[derive(GraphNode)]
struct Root {
    #[graphql_field(kind = "obj")]
    foo: Rc<Foo>,
}

impl Root {
    fn new() -> Self {
        Self {
            foo: Rc::new(Foo::new()),
        }
    }
}

fn main() {
    dbg!(Root::schema());

    let input = "query {
        foo {
            value
            name
            bar {
                value
            }
        }
    }";

    let query = Query::try_new(input).unwrap();
    dbg!(&query);

    let out = query.execute(Rc::new(Root::new()));
    println!("{}", out);
}
