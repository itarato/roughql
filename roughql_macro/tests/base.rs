use roughql_macro::GraphQLSource;

#[derive(Default, GraphQLSource)]
struct Foo {
    #[graphql_field(kind = "int")]
    val: i64,
}
