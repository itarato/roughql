use roughql_lib::{execute, parse, GraphObject, GraphPrimitiveType, GraphType};

struct Bar {}

impl Bar {
    fn value(&self) -> i64 {
        5678
    }
}

impl GraphObject for Bar {
    fn node_for(&self, name: String) -> GraphType {
        match name.as_str() {
            "value" => GraphType::Primitive(GraphPrimitiveType::Int(self.value())),
            _ => panic!("Cannot resolve Bar item: {}", name),
        }
    }
}

struct Foo {}

impl Foo {
    fn value(&self) -> i64 {
        1234
    }

    fn bar(&self) -> Bar {
        Bar {}
    }
}

impl GraphObject for Foo {
    fn node_for(&self, name: String) -> GraphType {
        match name.as_str() {
            "value" => GraphType::Primitive(GraphPrimitiveType::Int(self.value())),
            "bar" => GraphType::Object(Box::new(self.bar())),
            _ => panic!("Cannot resolve Foo item: {}", name),
        }
    }
}

struct Root;

impl GraphObject for Root {
    fn node_for(&self, name: String) -> GraphType {
        match name.as_str() {
            "foo" => GraphType::Object(Box::new(Foo {})),
            _ => panic!("Cannot resolve Root item: {}", name),
        }
    }
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

    let out = execute(query, Box::new(Root {}));
    println!("{}", out);
}

/***
 * How to:
 * - generate Root?
 * - get rid of field definition with procedural macros (maybe)?
 */
