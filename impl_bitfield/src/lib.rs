use std::{fmt::format, mem::Discriminant};

use proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use syn::{
    DataEnum, DeriveInput, Error, Ident, Result, Type, parse_macro_input,
    token::{Pub, Struct},
};

fn check_if_power_2(num: usize) -> bool {
    if (num & (num - 1)) != 0 {
        println!("Given number is not power of 2.");
        return false;
    } else {
        println!("Given number is power of 2.");
        return true;
    }
}

fn check_if_bool(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) if type_path.clone().into_token_stream().to_string() == "bool" => {
            true
        }
        _ => false,
    }
}

#[proc_macro_derive(BitfieldSpecifier)]
pub fn bitfield_specifier(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let result = expand(ast).unwrap_or_else(Error::into_compile_error);
    result.into()
}

fn expand(ast: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let data = ast.data;
    let enum_ident = ast.ident;
    eprintln!("data is {:#?}", data);
    let mut enum_elements: Vec<&Ident> = Vec::new();
    let first_enum_ident = match data {
        syn::Data::Enum(DataEnum { ref variants, .. }) => {
            for x in variants {
                eprintln!("variant's ident is {:#?}", x.ident);
                eprintln!("variant's discriminant is {:#?}", x.discriminant);

                let ident = &x.ident;
                enum_elements.push(ident);
            }
            eprintln!("variants[0]: {:#?}", variants[0].ident);
            &variants[0].ident
        }
        _ => unimplemented!("here1"),
    };

    let length = enum_elements.len();

    if !check_if_power_2(length) {
        let error = Err(syn::Error::new(
            // x.ident.span(),
            Span::call_site(),
            format!("BitfieldSpecifier expected a number of variants which is a power of 2"),
        ));
        return error;
    }

    let impl_check_discrinminant_range = enum_elements.iter().map(|ident| {
        let new_ident = format_ident!("{}_check", ident);
        quote! {
            fn #new_ident() {
                let _: <<[();(#enum_ident::#ident as usize)/#length] as MyTempTrait>::CCC as DiscriminantInRange>::PlaceHolder;
            }

        }
    });

    let impl_clone_inner = enum_elements.iter().map(|ident| {
        quote! {
            #enum_ident::#ident => #enum_ident::#ident,
        }
    });

    let impl_default_inner = enum_elements.iter().map(|ident| {
        quote! {
            if #enum_ident::#ident.discriminant() == 0 { return #enum_ident::#ident}
        }
    });

    let output1 = quote! {
        impl #enum_ident {
            fn discriminant(&self) -> u8 {
                unsafe { *<*const _>::from(self).cast::<u8>() }
            }
        }

        impl Clone for #enum_ident {
            fn clone(&self) -> #enum_ident {
                match self {
                    #(#impl_clone_inner)*
                }
            }
        }

        impl Default for #enum_ident {
            fn default() -> Self {
                #(#impl_default_inner)*
                else {
                    return #enum_ident::#first_enum_ident
                }
            }
        }

        impl Specifier for #enum_ident {
            const BITS:usize = 3;
            type AssocType = #enum_ident;
        }

        impl #enum_ident {
            fn get_value(value: &<#enum_ident as Specifier>::AssocType) -> <#enum_ident as Specifier>::AssocType {
                value.clone()
            }
        }

        #(#impl_check_discrinminant_range)*

    };

    Ok(output1)
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
        _ => unimplemented!("here2"),
    };

    let generate_empty_enum = (1usize..=64).map(|index| {
        let index_ident = Ident::new(&format!("B{}", index), Span::call_site());
        let impl_specifier = match index {
            1..=8 => quote! {
                impl Specifier for #index_ident {
                    const BITS:usize = #index;
                    type AssocType = u8;
                }
            },
            9..=16 => quote! {
                impl Specifier for #index_ident {
                    const BITS:usize = #index;
                    type AssocType = u16;
                }
            },
            17..=32 => quote! {
                impl Specifier for #index_ident {
                    const BITS:usize = #index;
                    type AssocType = u32;
                }
            },
            33..=64 => quote! {
                impl Specifier for #index_ident {
                    const BITS:usize = #index;
                    type AssocType = u64;
                }
            },
            _ => unimplemented!("here3"),
        };
        quote! {
            enum #index_ident {}

            #impl_specifier

            impl #index_ident {
                fn get_value(value: &<#index_ident as Specifier>::AssocType) -> <#index_ident as Specifier>::AssocType{
                    *value
                }
            }
        }
    });

    let mut ident_ty_list: Vec<(&Ident, &Type)> = Vec::new();

    let size_calc = fields.clone().into_iter().map(|field| {
        let ty = field.ty;
        if check_if_bool(&ty) {
            return quote! {+ 1};
        }
        quote! {+ <#ty as Specifier>::BITS}
    });

    let bit_length_stream = proc_macro2::TokenStream::from_iter(size_calc);

    let data_size = quote! {
        #[repr(C)]
        struct #ident {
            data: [u8; (0  #bit_length_stream) / 8]
        }
    };

    eprintln!("data_size is {}", data_size);

    for f in fields.iter() {
        let f_ident = f.ident.as_ref().unwrap();
        let f_ty = &f.ty;
        ident_ty_list.push((f_ident, f_ty));
    }

    let builder_fields = fields.iter().map(|f| {
        let f_ident = f.ident.as_ref().unwrap();
        let f_ty = &f.ty;
        if check_if_bool(f_ty) {
            return quote! { #f_ident:#f_ty,};
        }
        quote! {
            #f_ident:<#f_ty as Specifier>::AssocType,
        }
    });

    let builder_name = format_ident!("{}builder", ident);

    let builder_struct = quote! {
        struct #builder_name {
            #(#builder_fields)*
        }
    };

    let struct_inner = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        if check_if_bool(ty) {
            return quote! { #ident: false,};
        }

        quote! {
            #ident:<#ty as Specifier>::AssocType::default(), //here
        }
    });
    let builder_new = quote! {
        impl #ident {
            fn new() -> #builder_name {
                #builder_name {
                    #(#struct_inner)*
                }
            }
        }

    };
    eprintln!("builder_new is {}", builder_new);

    let set_get_inner = ident_ty_list.iter().map(|(ident, ty)| {
        let set_ident = format_ident!("set_{}", ident);
        let get_ident = format_ident!("get_{}", ident);
        if check_if_bool(ty) {
            return quote! {
                fn #set_ident(&mut self, value: #ty) {
                    self.#ident = value;
                }

                fn #get_ident(&self)-> #ty {
                    self.#ident
                }
            };
        }
        quote! {
            fn #set_ident(&mut self, value: <#ty as Specifier>::AssocType) {
                self.#ident = value;
            }

            fn #get_ident(&self)-> <#ty as Specifier>::AssocType {
                #ty::get_value(&self.#ident)
            }
        }
    });

    let get_set = quote! {
        impl #builder_name {
            #(#set_get_inner)*
        }
    };

    let output = quote! {

        #(#generate_empty_enum)*
        #data_size

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
