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
    #[serde(rename = "double")]
    Double,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "std::string")]
    StdString,
    #[serde(rename = "vec2")]
    Vec2,
    EntityID,
    #[serde(other)]
    Other,
}

impl Typ {
    fn as_rust_type(&self) -> proc_macro2::TokenStream {
        match self {
            Typ::Int => quote!(i32),
            Typ::UInt => quote!(u32),
            Typ::Float => quote!(f32),
            Typ::Double => quote!(f64),
            Typ::Bool => quote!(bool),
            Typ::StdString => quote!(Cow<'_, str>),
            Typ::Vec2 => quote! {(f32, f32)},
            Typ::EntityID => quote! { Option<EntityID> },
            Typ::Other => todo!(),
        }
    }
    fn as_rust_type_return(&self) -> proc_macro2::TokenStream {
        match self {
            Typ::StdString => quote! {Cow<'static, str>},
            Typ::EntityID => quote! {Option<EntityID>},
            _ => self.as_rust_type(),
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
    EntityID,
    #[serde(rename = "component_id")]
    ComponentID,
    #[serde(rename = "obj")]
    Obj,
    #[serde(rename = "color")]
    Color,
    #[serde(rename = "physics_body_id")]
    PhysicsBodyID,
}

impl Typ2 {
    fn as_rust_type(&self) -> proc_macro2::TokenStream {
        match self {
            Typ2::Int => quote! {i32},
            Typ2::Number => quote! {f64},
            Typ2::String => quote! {Cow<str>},
            Typ2::Bool => quote! {bool},
            Typ2::EntityID => quote! {EntityID},
            Typ2::ComponentID => quote!(ComponentID),
            Typ2::Obj => quote! {Obj},
            Typ2::Color => quote!(Color),
            Typ2::PhysicsBodyID => quote! {PhysicsBodyID},
        }
    }

    fn as_rust_type_return(&self) -> proc_macro2::TokenStream {
        match self {
            Typ2::String => quote! {Cow<'static, str>},
            Typ2::EntityID => quote! {Option<EntityID>},
            Typ2::ComponentID => quote!(Option<ComponentID>),
            _ => self.as_rust_type(),
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
    typ: Typ2,
    default: Option<String>,
}

#[derive(Deserialize)]
struct FnRet {
    // name: String,
    typ: Typ2,
    optional: bool,
    is_vec: bool,
}
impl FnRet {
    fn as_rust_type_return(&self) -> proc_macro2::TokenStream {
        let mut ret = self.typ.as_rust_type_return();
        if self.is_vec {
            ret = quote! {
                Vec<#ret>
            };
        }
        if self.optional {
            ret = quote! {
                Option<#ret>
            };
        }
        ret
    }
}

#[derive(Deserialize)]
struct ApiFn {
    fn_name: String,
    desc: String,
    args: Vec<FnArg>,
    rets: Vec<FnRet>,
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
        let field_name_raw = &field.field;
        let field_name_s = convert_field_name(&field.field);
        let field_name = format_ident!("{}", field_name_s);
        let field_doc = &field.desc;
        let set_method_name = format_ident!("set_{}", field_name);
        match field.typ {
            Typ::Int
            | Typ::UInt
            | Typ::Float
            | Typ::Double
            | Typ::Bool
            | Typ::Vec2
            | Typ::EntityID
            | Typ::StdString => {
                let field_type = field.typ.as_rust_type();
                let field_type_ret = field.typ.as_rust_type_return();
                Some(quote! {
                    #[doc = #field_doc]
                    pub fn #field_name(self) -> eyre::Result<#field_type_ret> {
                        // This trasmute is used to reinterpret i32 as u32 in one case.
                        raw::component_get_value(self.0, #field_name_raw)
                     }
                    #[doc = #field_doc]
                    pub fn #set_method_name(self, value: #field_type) -> eyre::Result<()> {
                        raw::component_set_value(self.0, #field_name_raw, value)
                     }
                })
            }
            _ => None,
        }
    });

    let com_name = com.name;

    quote! {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct #component_name(pub ComponentID);

        impl Component for #component_name {
            const NAME_STR: &'static str = #com_name;
        }

        impl From<ComponentID> for #component_name {
            fn from(com: ComponentID) -> Self {
                #component_name(com)
            }
        }

        impl From<#component_name> for ComponentID {
            fn from(com: #component_name) -> Self {
                com.0
            }
        }

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
        let optional = arg.default.is_some();
        if optional {
            quote! {
                #arg_name: Option<#arg_type>
            }
        } else {
            quote! {
                #arg_name: #arg_type
            }
        }
    });

    let put_args_pre = api_fn.args.iter().enumerate().map(|(i, arg)| {
        let arg_name = format_ident!("{}", arg.name);
        let i = i as i32;
        quote! {
            if LuaPutValue::is_non_empty(&#arg_name) {
                last_non_empty = #i;
            }
        }
    });

    let put_args = api_fn.args.iter().enumerate().map(|(i, arg)| {
        let arg_name = format_ident!("{}", arg.name);
        let i = i as i32;
        quote! {
            if #i <= last_non_empty {
                LuaPutValue::put(&#arg_name, lua);
            }
        }
    });

    let ret_type = if api_fn.rets.is_empty() {
        quote! { () }
    } else {
        if api_fn.rets.len() == 1 {
            let ret = api_fn.rets.first().unwrap();
            ret.as_rust_type_return()
        } else {
            let ret_types = api_fn.rets.iter().map(|ret| ret.as_rust_type_return());
            quote! { ( #(#ret_types),* ) }
        }
    };

    let fn_name_c = name_to_c_literal(api_fn.fn_name);

    let ret_count = api_fn.rets.len() as i32;

    quote! {
        #[doc = #fn_doc]
        pub fn #fn_name(#(#args,)*) -> eyre::Result<#ret_type> {
            let lua = LuaState::current()?;

            lua.get_global(#fn_name_c);

            let mut last_non_empty: i32 = -1;
            #(#put_args_pre)*
            #(#put_args)*

            lua.call(last_non_empty+1, #ret_count);

            let ret = LuaGetValue::get(lua, -1);
            if #ret_count > 0 {
                lua.pop_last_n(#ret_count);
            }
            ret
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
    let fn_name_c = name_to_c_literal(fn_name);
    quote! {
        unsafe extern "C" fn #bridge_fn_name(lua: *mut lua_State) -> c_int {
            let lua_state = noita_api::lua::LuaState::new(lua);
            lua_state.make_current();
            noita_api::lua::LuaFnRet::do_return(#fn_name_ident(lua_state), lua_state)
        }

        LUA.lua_pushcclosure(lua, Some(#bridge_fn_name), 0);
        LUA.lua_setfield(lua, -2, #fn_name_c.as_ptr());
    }
    .into()
}

fn name_to_c_literal(name: String) -> proc_macro2::Literal {
    proc_macro2::Literal::c_string(CString::new(name).unwrap().as_c_str())
}
