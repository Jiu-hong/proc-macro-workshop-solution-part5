use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{
    Ident, Token,
    parse::Parse,
    parse_macro_input,
    token::{Pub, Struct},
};
// #[derive(Debug)]
struct MyStruct {
    pub_kw: Pub,
    struct_kw: Struct,
    struct_name: Ident,
    b1: Ident,
    b3: Ident,
    b4: Ident,
    b24: Ident,
}

impl Parse for MyStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pub_kw = input.parse::<Token![pub]>()?;
        let struct_kw = input.parse::<Token![struct]>()?;
        let struct_name: Ident = input.parse()?;
        let content;
        let _ = syn::braced!(content in input);
        let _: Ident = content.parse()?;
        let _ = content.parse::<Token![:]>()?;
        let b1: Ident = content.parse()?;
        let _ = content.parse::<Token![,]>()?;
        let _: Ident = content.parse()?;
        let _ = content.parse::<Token![:]>()?;
        let b3: Ident = content.parse()?;
        let _ = content.parse::<Token![,]>()?;
        let _: Ident = content.parse()?;
        let _ = content.parse::<Token![:]>()?;
        let b4: Ident = content.parse()?;
        let _ = content.parse::<Token![,]>()?;
        let _: Ident = content.parse()?;
        let _ = content.parse::<Token![:]>()?;
        let b24: Ident = content.parse()?;
        let _ = content.parse::<Token![,]>()?;

        Ok(MyStruct {
            pub_kw,
            struct_kw,
            struct_name,
            b1,
            b3,
            b4,
            b24,
        })
    }
}

fn extract_number(ident_str: &str) -> usize {
    let re = Regex::new("[0-9]+").unwrap();
    let m: regex::Match<'_> = re.find(&ident_str).unwrap();

    m.as_str().parse::<usize>().unwrap()
}
#[proc_macro_attribute]
pub fn bitfield(
    _: proc_macro::TokenStream,
    input2: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    eprintln!("input2: {:#?}", input2);
    let input3_clone = input2.clone();
    let my_struct: MyStruct = parse_macro_input!(input3_clone);

    let pub_kw = my_struct.pub_kw;
    let struct_kw = my_struct.struct_kw;
    let struct_name = my_struct.struct_name;

    let b1 = my_struct.b1;
    let b1_str = b1.to_string();
    let b3 = my_struct.b3;
    let b3_str = b3.to_string();
    let b4 = my_struct.b4;
    let b4_str = b4.to_string();
    let b24 = my_struct.b24;
    let b24_str = b24.to_string();

    let matched_int_b1 = extract_number(&b1_str);
    let matched_int_b3 = extract_number(&b3_str);
    let matched_int_b4 = extract_number(&b4_str);
    let matched_int_b24 = extract_number(&b24_str);

    let size = (matched_int_b1 + matched_int_b3 + matched_int_b4 + matched_int_b24) / 8;

    let output = quote! {
        enum #b1 {}
        impl Specifier for #b1 {
            const BITS: usize = #matched_int_b1;
        }
        enum #b3 {}
        impl Specifier for #b3 {
            const BITS: usize = #matched_int_b3;
        }
        enum #b4 {}
        impl Specifier for #b4 {
            const BITS: usize = #matched_int_b4;
        }
        enum #b24 {}
        impl Specifier for #b24 {
            const BITS: usize = #matched_int_b24;
        }
    };

    let output_struct = quote! {
        #[repr(C)]
        #pub_kw #struct_kw #struct_name {
            data: [u8; #size],
        }
    };

    quote! {#output_struct #output }.into()
}
