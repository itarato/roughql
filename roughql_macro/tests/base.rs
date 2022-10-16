use roughql_macro::GraphNode;

#[derive(Default, GraphNode)]
struct Foo {
    #[graphql_field(kind = "int")]
    val: i64,
}
