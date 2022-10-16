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
    Object(Box<dyn GraphObject>),
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
pub struct Query(Vec<QueryField>);
