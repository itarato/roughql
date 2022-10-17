use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

#[derive(Debug)]
pub enum GraphPrimitiveType {
    Int(i64),
    Str(String),
}

impl GraphPrimitiveType {
    pub fn to_string(&self) -> String {
        match self {
            GraphPrimitiveType::Int(v) => format!("{}", v),
            GraphPrimitiveType::Str(v) => format!("\"{}\"", v),
        }
    }
}

pub enum GraphNodeType {
    Primitive(GraphPrimitiveType),
    Compound(Rc<dyn GraphNodeProvider>),
}

pub trait GraphNodeProvider {
    fn node_for(&self, name: String) -> GraphNodeType;
}

#[derive(Debug)]
pub enum SchemaPrimitiveType {
    Int,
    Str,
}

#[derive(Debug)]
pub enum Schema {
    Leaf(SchemaPrimitiveType),
    Node(HashMap<String, Schema>),
}

pub trait SchemaProvider {
    fn schema() -> Schema;
}

#[derive(Debug)]
pub enum QueryField {
    Leaf(String),
    Node((String, Vec<QueryField>)),
}

#[derive(Debug)]
pub struct Query(pub Vec<QueryField>);

impl Query {
    pub fn try_new<'a>(query: &str) -> Result<Self, &'a str> {
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

    pub fn execute(self, root: Rc<dyn GraphNodeProvider>) -> String {
        let mut out = String::new();

        out.push_str("{\"data\": { ");

        for field in self.0 {
            let field_out = build_query_result(field, root.clone());
            out.push_str(&field_out);
        }

        out.push_str(" } }");

        out
    }
}

fn build_query_result(query_field: QueryField, source: Rc<dyn GraphNodeProvider>) -> String {
    let mut out: String = String::new();

    match query_field {
        QueryField::Leaf(name) => match source.node_for(name.clone()) {
            GraphNodeType::Primitive(val) => {
                out.push_str(&format!("\"{}\": ", name));
                out.push_str(&val.to_string());
            }
            GraphNodeType::Compound(_) => panic!("Expected primitive, got object for {}", name),
        },

        QueryField::Node((name, children)) => {
            out.push_str(&format!("\"{}\": {{ ", name));

            let target = match source.node_for(name.clone()) {
                GraphNodeType::Primitive(_) => {
                    panic!("Expected object, got primitive for {}", name)
                }
                GraphNodeType::Compound(o) => o.clone(),
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
                    Some((new_tokens, QueryField::Node((name.to_string(), fields))))
                } else {
                    None
                }
            })
            .ok_or("Missing list ending `}`")
    } else {
        Ok((tokens, QueryField::Leaf(name.to_string())))
    }
}
