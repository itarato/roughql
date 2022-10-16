use std::{collections::VecDeque, rc::Rc};

#[derive(Debug)]
enum GraphPrimitiveType {
    Int(i64),
}

impl GraphPrimitiveType {
    fn to_string(&self) -> String {
        match self {
            GraphPrimitiveType::Int(v) => format!("{}", v),
        }
    }
}

enum GraphType {
    Primitive(GraphPrimitiveType),
    Object(Box<dyn GraphObject>),
}

trait GraphObject {
    fn node_for(&self, name: String) -> GraphType;
}

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

#[derive(Debug)]
enum QueryField {
    Leaf(String),
    Object((String, Vec<QueryField>)),
}

#[derive(Debug)]
struct Query(Vec<QueryField>);

struct Root;

impl GraphObject for Root {
    fn node_for(&self, name: String) -> GraphType {
        match name.as_str() {
            "foo" => GraphType::Object(Box::new(Foo {})),
            _ => panic!("Cannot resolve Root item: {}", name),
        }
    }
}

impl Root {
    fn build_query_result(query_field: QueryField, source: Rc<Box<dyn GraphObject>>) -> String {
        let mut out: String = String::new();

        match query_field {
            QueryField::Leaf(name) => match source.node_for(name.clone()) {
                GraphType::Primitive(val) => {
                    out.push_str(&format!("\"{}\": ", name));
                    out.push_str(&val.to_string());
                }
                GraphType::Object(_) => panic!("Expected primitive, got object for {}", name),
            },

            QueryField::Object((name, children)) => {
                out.push_str(&format!("\"{}\": {{ ", name));

                let target = match source.node_for(name.clone()) {
                    GraphType::Primitive(_) => {
                        panic!("Expected object, got primitive for {}", name)
                    }
                    GraphType::Object(o) => Rc::new(o),
                };

                let mut is_first: bool = true;
                for child in children {
                    if !is_first {
                        out.push_str(", ");
                    }

                    let child_out = Self::build_query_result(child, target.clone());
                    out.push_str(&child_out);

                    is_first = false;
                }

                out.push_str(" }");
            }
        }

        out
    }

    fn execute(query: Query) -> String {
        let mut out = String::new();

        out.push_str("{\"data\": { ");

        for field in query.0 {
            let field_out = Self::build_query_result(field, Rc::new(Box::new(Self {})));
            out.push_str(&field_out);
        }

        out.push_str(" } }");

        out
    }

    fn parse<'a>(query: &str) -> Result<Query, &'a str> {
        let mut tokens = query
            .split(' ')
            .map(|slice| slice.trim())
            .filter(|&slice| !slice.is_empty())
            .collect::<VecDeque<&str>>();

        match tokens.pop_front() {
            Some("query") => match tokens.pop_front() {
                Some("{") => match Self::parse_list(tokens) {
                    // This is missing checking the last closing `}`.
                    Ok((mut new_tokens, query_field)) => match new_tokens.pop_front() {
                        Some("}") => Ok(Query(query_field)),
                        _ => Err("Invalid ending"),
                    },
                    Err(e) => Err(e),
                },
                _ => Err("Invalid query start"),
            },
            _ => Err("Unexpected query type"),
        }
    }

    fn parse_list<'a>(
        mut tokens: VecDeque<&str>,
    ) -> Result<(VecDeque<&str>, Vec<QueryField>), &'a str> {
        let mut query_fields: Vec<QueryField> = vec![];

        loop {
            match tokens.front() {
                Some(&"}") => return Ok((tokens, query_fields)),
                Some(_) => {
                    match Self::parse_elem(tokens) {
                        Ok((new_tokens, field)) => {
                            tokens = new_tokens;
                            query_fields.push(field);
                        }
                        Err(e) => return Err(e),
                    };
                }
                None => return Err("Invalid end of list"),
            };
        }
    }

    fn parse_elem<'a>(mut tokens: VecDeque<&str>) -> Result<(VecDeque<&str>, QueryField), &'a str> {
        let name = tokens.pop_front().ok_or("Missing name")?;

        if let Some(&"{") = tokens.front() {
            tokens.pop_front();

            let (mut new_tokens, fields) = Self::parse_list(tokens)?;

            new_tokens
                .pop_front()
                .and_then(|s| {
                    if s == "}" {
                        Some((new_tokens, QueryField::Object((name.to_string(), fields))))
                    } else {
                        None
                    }
                })
                .ok_or("Missing list ending `}`")
        } else {
            Ok((tokens, QueryField::Leaf(name.to_string())))
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

    let query = Root::parse(input).unwrap();
    dbg!(&query);

    let out = Root::execute(query);
    println!("{}", out);
}

/***
 * How to:
 * - generate Root?
 * - get rid of field definition with procedural macros (maybe)?
 */
