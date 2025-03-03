use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use regex::Regex;
use syn::{
    DeriveInput, Ident, Token,
    parse::Parse,
    parse_macro_input,
    token::{Pub, Struct},
};
// #[derive(Debug)]
struct MyStruct {
    pub_kw: Pub,
    struct_kw: Struct,
    struct_name: Ident,
    a: Ident,
    b1: Ident,
    b: Ident,
    b3: Ident,
    c: Ident,
    b4: Ident,
    d: Ident,
    b24: Ident,
}

impl Parse for MyStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pub_kw = input.parse::<Token![pub]>()?;
        let struct_kw = input.parse::<Token![struct]>()?;
        let struct_name: Ident = input.parse()?;
        let content;
        let _ = syn::braced!(content in input);
        let a: Ident = content.parse()?;
        let _ = content.parse::<Token![:]>()?;
        let b1: Ident = content.parse()?;
        let _ = content.parse::<Token![,]>()?;
        let b: Ident = content.parse()?;
        let _ = content.parse::<Token![:]>()?;
        let b3: Ident = content.parse()?;
        let _ = content.parse::<Token![,]>()?;
        let c: Ident = content.parse()?;
        let _ = content.parse::<Token![:]>()?;
        let b4: Ident = content.parse()?;
        let _ = content.parse::<Token![,]>()?;
        let d: Ident = content.parse()?;
        let _ = content.parse::<Token![:]>()?;
        let b24: Ident = content.parse()?;
        let _ = content.parse::<Token![,]>()?;

        Ok(MyStruct {
            pub_kw,
            struct_kw,
            struct_name,
            a,
            b1,
            b,
            b3,
            c,
            b4,
            d,
            b24,
        })
    }
}

// fn extract_number(ident_str: &str) -> usize {
//     let new_ident_str = match ident_str {
//         "A" => "B1",
//         "B" => "B3",
//         "C" => "B4",
//         "D" => "B24",
//         _ => ident_str,
//     };

//     let re = Regex::new("[0-9]+").unwrap();
//     println!("Herehere");
//     let m: regex::Match<'_> = re.find(&new_ident_str).unwrap();

//     m.as_str().parse::<usize>().unwrap()
// }

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

    let a = my_struct.a;
    let b1 = my_struct.b1;
    let b1_str = b1.to_string();
    let b = my_struct.b;
    let b3 = my_struct.b3;
    let b3_str = b3.to_string();
    let c = my_struct.c;
    let b4 = my_struct.b4;
    let b4_str = b4.to_string();
    let d = my_struct.d;
    let b24 = my_struct.b24;
    let b24_str = b24.to_string();

    println!("hello");

    // let size = (matched_int_b1 + matched_int_b3 + matched_int_b4 + matched_int_b24) / 8;

    let b_type_iterator = (1..=64).map(|number| {
        let b_type = Ident::new(&format!("B{}", number), Span::call_site());
        quote! {
            enum #b_type {}
            impl Specifier for #b_type {
                const BITS: usize = #number;
                type AssocType = u64;
            }
        }
    });
    let output = proc_macro2::TokenStream::from_iter(b_type_iterator);

    let builder_name = format_ident!("{}builder", struct_name);

    let builder_struct = quote! {
        struct #builder_name {
            #a:<#b1 as Specifier>::AssocType,
            #b:<#b3 as Specifier>::AssocType,
            #c:<#b4 as Specifier>::AssocType,
            #d:<#b24 as Specifier>::AssocType,
        }
    };

    let output_struct = quote! {
        #[repr(C)]
        #pub_kw #struct_kw #struct_name {
            data: [u8; <#b1 as Specifier>::BITS + <#b3 as Specifier>::BITS +<#b4 as Specifier>::BITS <#b24 as Specifier>::BITS ],
        }

        impl #struct_name {
            fn new() -> #builder_name {
                #builder_name {
                    #a:<#b1 as Specifier>::AssocType::default(),
                    #b:<#b3 as Specifier>::AssocType::default(),
                    #c:<#b4 as Specifier>::AssocType::default(),
                    #d:<#b24 as Specifier>::AssocType::default(),
                }
            }
        }

        impl #builder_name {
            fn get_a(&self) -> <#b1 as Specifier>::AssocType{
                self.a
            }
            fn set_a(&mut self,value: <#b1 as Specifier>::AssocType) {
                self.a=value
            }
            fn get_b(&self) -> <#b3 as Specifier>::AssocType{
                self.b
            }
            fn set_b(&mut self,value: <#b3 as Specifier>::AssocType) {
                self.b=value
            }
            fn get_c(&self) -> <#b4 as Specifier>::AssocType{
                self.c
            }

            fn set_c(&mut self,value: <#b4 as Specifier>::AssocType) {
                self.c=value
            }

            fn get_d(&self) -> <#b24 as Specifier>::AssocType{
                self.d
            }

            fn set_d(&mut self,value: <#b24 as Specifier>::AssocType) {
                self.d=value
            }
        }
    };

    quote! {
    #builder_struct
    #output_struct
    #output }
    .into()
}
