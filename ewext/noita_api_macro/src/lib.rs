use std::ffi::CString;

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
enum Typ2 {
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "entity_id")]
    EntityId,
    #[serde(rename = "component_id")]
    ComponentId,
    #[serde(rename = "obj")]
    Obj,
    #[serde(rename = "color")]
    Color,
    // #[serde(other)]
    // Other,
}

impl Typ2 {
    fn as_rust_type(&self) -> proc_macro2::TokenStream {
        match self {
            Typ2::Int => quote! {i32},
            Typ2::Number => quote! {f64},
            Typ2::String => quote! {String},
            Typ2::Bool => quote! {bool},
            Typ2::EntityId => quote! {EntityID},
            Typ2::ComponentId => quote!(ComponentID),
            Typ2::Obj => quote! {Obj},
            Typ2::Color => quote!(Color),
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

#[derive(Deserialize)]
struct FnArg {
    name: String,
    typ: Typ2, // TODO
    default: Option<String>,
}

#[derive(Deserialize)]
struct ApiFn {
    fn_name: String,
    desc: String,
    args: Vec<FnArg>,
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
                    #[doc = #field_doc]
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

fn generate_code_for_api_fn(api_fn: ApiFn) -> proc_macro2::TokenStream {
    let fn_name = format_ident!("{}", api_fn.fn_name.to_snek_case());
    let fn_doc = api_fn.desc;

    let args = api_fn.args.iter().map(|arg| {
        let arg_name = format_ident!("{}", arg.name);
        let arg_type = arg.typ.as_rust_type();
        quote! {
            #arg_name: #arg_type
        }
    });

    quote! {
        #[doc = #fn_doc]
        pub(crate) fn #fn_name(#(#args,)*) {

        }
    }
}

#[proc_macro]
pub fn generate_api(_item: TokenStream) -> TokenStream {
    let api_fns: Vec<ApiFn> = serde_json::from_str(include_str!("lua_api.json")).unwrap();

    let res = api_fns.into_iter().map(generate_code_for_api_fn);
    quote! {#(#res)*}.into()
}

#[proc_macro]
pub fn add_lua_fn(item: TokenStream) -> TokenStream {
    let mut tokens = item.into_iter();

    let fn_name = tokens.next().unwrap().to_string();
    let fn_name_ident = format_ident!("{fn_name}");
    let bridge_fn_name = format_ident!("{fn_name}_lua_bridge");
    let fn_name_c = proc_macro2::Literal::c_string(CString::new(fn_name).unwrap().as_c_str());
    quote! {
        unsafe extern "C" fn #bridge_fn_name(lua: *mut lua_State) -> c_int {
            let lua_state = LuaState::new(lua);
            lua_state.make_current();
            crate::lua_state::LuaFnRet::do_return(#fn_name_ident(lua_state), lua_state)
        }

        LUA.lua_pushcclosure(lua, Some(#bridge_fn_name), 0);
        LUA.lua_setfield(lua, -2, #fn_name_c.as_ptr());
    }
    .into()
}
