use proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use syn::{
    DataEnum, DeriveInput, Ident, Token, Type, TypePath,
    parse::Parse,
    parse_macro_input,
    token::{Pub, Struct},
};

#[proc_macro_derive(BitfieldSpecifier)]
pub fn bitfield_specifier(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let data = ast.data;
    eprintln!("data is {:#?}", data);
    let mut enum_elements: Vec<(&str, &Ident)> = Vec::new();
    let first_enum_ident = match data {
        syn::Data::Enum(DataEnum { ref variants, .. }) => {
            for x in variants {
                eprintln!("variant's ident is {:#?}", x.ident);
                eprintln!("variant's discriminant is {:#?}", x.discriminant);
                let a = &x.discriminant.as_ref().unwrap().1;

                let str = match a {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(number),
                        ..
                    }) => number.base10_digits(),
                    _ => unimplemented!(),
                };
                let ident = &x.ident;
                enum_elements.push((str, ident));
            }
            eprintln!("variants[0]: {:#?}", variants[0].ident);
            &variants[0].ident
        }
        _ => unimplemented!(),
    };
    let enum_ident = ast.ident;
    let impl_into_inner = enum_elements.iter().map(|(str, ident)| {
        let number: u8 = str.parse().unwrap();
        quote! {
            #number =>  #enum_ident::#ident,
        }
    });
    quote! {
        impl TriggerMode {
            fn discriminant(&self) -> u8 {
                // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
                // between `repr(C)` structs, each of which has the `u8` discriminant as its first
                // field, so we can read the discriminant without offsetting the pointer.
                unsafe { *<*const _>::from(self).cast::<u8>() }
            }
        }

        impl Into<#enum_ident> for u8 {
            fn into(self) -> #enum_ident {
                match self {
                    #(#impl_into_inner)*
                    _ => #enum_ident::#first_enum_ident,
                }
            }
        }


    }
    .into()
}

#[proc_macro_attribute]
pub fn bitfield(
    _: proc_macro::TokenStream,
    input2: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    eprintln!("input2: {:#?}", input2);
    let input3_clone = input2.clone();

    let ast: DeriveInput = parse_macro_input!(input3_clone);
    // input2;
    eprintln!("ast is {:#?}", ast);
    let ident = ast.ident;

    let data = ast.data;
    let fields = match data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => fields,
        _ => unimplemented!(),
    };

    let mut ident_ty_list: Vec<(&Ident, &Type)> = Vec::new();

    for f in fields.iter() {
        let f_ident = f.ident.as_ref().unwrap();
        let f_ty = &f.ty;
        ident_ty_list.push((f_ident, f_ty));
    }

    let builder_fields = fields.iter().map(|f| {
        let f_ident = f.ident.as_ref().unwrap();
        let f_ty = &f.ty;
        quote! {
            #f_ident:#f_ty,
        }
    });

    let builder_fields_clone1 = builder_fields.clone();

    let builder_name = format_ident!("{}builder", ident);

    let builder_struct = quote! {
        struct #builder_name {
            #(#builder_fields)*
        }
    };

    let builder_new = quote! {
        impl #ident {
            fn new() -> #builder_name {
                #(#builder_fields_clone1)*
            }
        }
    };

    let set_get_inner = ident_ty_list.iter().map(|(ident, ty)| {
        let set_ident = format_ident!("set_{}", ident);
        let get_ident = format_ident!("get_{}", ident);
        quote! {
            fn #set_ident(&mut self, value: #ty) {
                self.#ident = value;
            }

            fn #get_ident(&self)-> #ty {
                let u8_value = self.#ident.discriminant();
                let trigger_mode_value: #ty = u8_value.into();
                trigger_mode_value
            }
        }
    });

    let get_set = quote! {
        impl #builder_name {
            #(#set_get_inner)*
        }
    };

    // eprintln!("builder_struct is {}", builder_struct);
    let output = quote! {

        // builder struct
        #builder_struct

        // builder new
        #builder_new

        //impl builder set & get
        #get_set


    };

    // input2
    output.into()
}
