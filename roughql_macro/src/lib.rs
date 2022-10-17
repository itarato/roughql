use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Data, DeriveInput, Field, Lit, Meta,
    MetaList, MetaNameValue, NestedMeta, Path, PathArguments, PathSegment, Type, TypePath,
};

enum FieldAttrType {
    PrimitiveInt,
    PrimitiveStr,
    Object,
}

struct FieldAttr {
    ty: FieldAttrType,
}

fn find_graphql_field_attr(field: &Field) -> Option<FieldAttr> {
    let attr = field.attrs.iter().find(|&attr| {
        attr.path
            .get_ident()
            .unwrap()
            .to_string()
            .eq("graphql_field")
    });

    if let Some(attr) = attr {
        if let Meta::List(MetaList { nested, .. }) = attr.parse_meta().unwrap() {
            if let Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                lit: Lit::Str(lit_str),
                ..
            }))) = nested.first()
            {
                return match lit_str.value().as_str() {
                    "int" => Some(FieldAttr {
                        ty: FieldAttrType::PrimitiveInt,
                    }),
                    "str" => Some(FieldAttr {
                        ty: FieldAttrType::PrimitiveStr,
                    }),
                    "obj" => Some(FieldAttr {
                        ty: FieldAttrType::Object,
                    }),
                    _ => None,
                };
            }
        }
    }

    None
}

fn rc_inner_type(ty: &Type) -> &Type {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(PathSegment {
            arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
            ..
        }) = segments.first()
        {
            if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
                return inner_ty;
            }
        }
    }

    panic!("Failed finding inner type of Rc<?>")
}

#[proc_macro_derive(GraphNode, attributes(graphql_field))]
pub fn derive_graphql_source_attr(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let struct_ident = input.ident;

    if let Data::Struct(ref data_struct) = input.data {
        let mut graphql_field_defs = vec![];
        let mut schema_field_defs = vec![];

        for field in &data_struct.fields {
            if let Some(field_attr) = find_graphql_field_attr(field) {
                let field_ident = &field.ident;
                let field_ident_string = field_ident.clone().unwrap().to_string();

                let graphql_field_def = match field_attr.ty {
                    FieldAttrType::PrimitiveInt => quote! {
                        #field_ident_string => roughql_lib::GraphNodeType::Primitive(roughql_lib::GraphPrimitiveType::Int(self.#field_ident))
                    },
                    FieldAttrType::PrimitiveStr => quote! {
                        #field_ident_string => roughql_lib::GraphNodeType::Primitive(roughql_lib::GraphPrimitiveType::Str(self.#field_ident.clone()))
                    },
                    FieldAttrType::Object => quote! {
                        #field_ident_string => roughql_lib::GraphNodeType::Compound(self.#field_ident.clone())
                    },
                };
                graphql_field_defs.push(graphql_field_def);

                let schema_field_def = match field_attr.ty {
                    FieldAttrType::PrimitiveInt => quote! {
                        (#field_ident_string.to_owned(), roughql_lib::Schema::Leaf(roughql_lib::SchemaPrimitiveType::Int))
                    },
                    FieldAttrType::PrimitiveStr => quote! {
                        (#field_ident_string.to_owned(), roughql_lib::Schema::Leaf(roughql_lib::SchemaPrimitiveType::Str))
                    },
                    FieldAttrType::Object => {
                        let inner_type = rc_inner_type(&field.ty);
                        quote! {
                           (#field_ident_string.to_owned(), #inner_type::schema())
                        }
                    }
                };
                schema_field_defs.push(schema_field_def);
            }
        }

        let panic_msg = format!("Cannot resolve {} item: {{}}", struct_ident.to_string());

        let impl_block = quote! {
            impl roughql_lib::GraphNodeProvider for #struct_ident {
                fn node_for(&self, name: std::string::String) -> roughql_lib::GraphNodeType {
                    match name.as_str() {
                        #(#graphql_field_defs,)*
                        _ => panic!(#panic_msg, name),
                    }
                }
            }

            impl roughql_lib::SchemaProvider for #struct_ident {
                fn schema() -> roughql_lib::Schema {
                    roughql_lib::Schema::Node(std::collections::HashMap::from([
                        #(#schema_field_defs,)*
                    ]))
                }
            }
        };

        return TokenStream::from(impl_block);
    }

    TokenStream::new()
}
