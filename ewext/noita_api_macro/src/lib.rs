use heck::ToSnekCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
enum Typ {
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "uint32")]
    UInt,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "std::string")]
    StdString,
    #[serde(rename = "vec2")]
    Vec2,
    #[serde(other)]
    Other,
}

impl Typ {
    fn as_rust_type(&self) -> proc_macro2::TokenStream {
        match self {
            Typ::Int => quote!(i32),
            Typ::UInt => quote!(u32),
            Typ::Float => quote!(f32),
            Typ::Bool => quote!(bool),
            Typ::StdString => todo!(),
            Typ::Vec2 => todo!(),
            Typ::Other => todo!(),
        }
    }
}

#[derive(Deserialize)]
struct Field {
    field: String,
    typ: Typ,
    desc: String,
}

#[derive(Deserialize)]
struct Component {
    name: String,
    fields: Vec<Field>,
}
#[proc_macro]
pub fn generate_components(_item: TokenStream) -> TokenStream {
    let components: Vec<Component> = serde_json::from_str(include_str!("components.json")).unwrap();

    let res = components.into_iter().map(generate_code_for_component);
    quote! {#(#res)*}.into()
}

fn convert_field_name(field_name: &str) -> String {
    if field_name == "type" {
        return "type_fld".to_owned();
    }
    if field_name == "loop" {
        return "loop_fld".to_owned();
    }
    field_name.to_snek_case()
}

fn generate_code_for_component(com: Component) -> proc_macro2::TokenStream {
    let component_name = format_ident!("{}", com.name);

    let impls = com.fields.iter().filter_map(|field| {
        let field_name = format_ident!("{}", convert_field_name(&field.field));
        let field_doc = &field.desc;
        match field.typ {
            Typ::Int | Typ::UInt | Typ::Float | Typ::Bool => {
                let field_type = field.typ.as_rust_type();
                let set_method_name = format_ident!("set_{}", field_name);
                Some(quote! {
                    #[doc = #field_doc]
                    fn #field_name(self) -> #field_type { todo!() }
                    fn #set_method_name(self, value: #field_type) { todo!() }
                })
            }
            _ => None,
        }
    });

    quote! {
        #[derive(Clone, Copy, PartialEq, Eq)]
        struct #component_name(u32);

        impl #component_name {
            #(#impls)*
        }
    }
}
