use std::{collections::VecDeque, rc::Rc};

#[derive(Debug)]
pub enum GraphPrimitiveType {
    Int(i64),
}

impl GraphPrimitiveType {
    pub fn to_string(&self) -> String {
        match self {
            GraphPrimitiveType::Int(v) => format!("{}", v),
        }
    }
}

pub enum GraphType {
    Primitive(GraphPrimitiveType),
    Compound(Rc<dyn GraphObject>),
}

pub trait GraphObject {
    fn node_for(&self, name: String) -> GraphType;
}

#[derive(Debug)]
pub enum QueryField {
    Leaf(String),
    Object((String, Vec<QueryField>)),
}

#[derive(Debug)]
pub struct Query(pub Vec<QueryField>);

fn build_query_result(query_field: QueryField, source: Rc<dyn GraphObject>) -> String {
    let mut out: String = String::new();

    match query_field {
        QueryField::Leaf(name) => match source.node_for(name.clone()) {
            GraphType::Primitive(val) => {
                out.push_str(&format!("\"{}\": ", name));
                out.push_str(&val.to_string());
            }
            GraphType::Compound(_) => panic!("Expected primitive, got object for {}", name),
        },

        QueryField::Object((name, children)) => {
            out.push_str(&format!("\"{}\": {{ ", name));

            let target = match source.node_for(name.clone()) {
                GraphType::Primitive(_) => {
                    panic!("Expected object, got primitive for {}", name)
                }
                GraphType::Compound(o) => o.clone(),
            };

            let mut is_first: bool = true;
            for child in children {
                if !is_first {
                    out.push_str(", ");
                }

                let child_out = build_query_result(child, target.clone());
                out.push_str(&child_out);

                is_first = false;
            }

            out.push_str(" }");
        }
    }

    out
}

pub fn execute(query: Query, root: Rc<dyn GraphObject>) -> String {
    let mut out = String::new();

    out.push_str("{\"data\": { ");

    for field in query.0 {
        let field_out = build_query_result(field, root.clone());
        out.push_str(&field_out);
    }

    out.push_str(" } }");

    out
}

pub fn parse<'a>(query: &str) -> Result<Query, &'a str> {
    let mut tokens = query
        .split(' ')
        .map(|slice| slice.trim())
        .filter(|&slice| !slice.is_empty())
        .collect::<VecDeque<&str>>();

    match tokens.pop_front() {
        Some("query") => match tokens.pop_front() {
            Some("{") => match parse_list(tokens) {
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
                match parse_elem(tokens) {
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

        let (mut new_tokens, fields) = parse_list(tokens)?;

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
