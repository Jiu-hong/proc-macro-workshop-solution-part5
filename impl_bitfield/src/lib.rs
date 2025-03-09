use proc_macro2::{Literal, Punct, Span};
use quote::{ToTokens, format_ident, quote};
use syn::{
    DataEnum, DeriveInput, Error, Ident, Result,  Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

fn check_if_bool(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) if type_path.clone().into_token_stream().to_string() == "bool" => {
            true
        }
        _ => false,
    }
}

fn logarithm_two(x: usize) -> Option<usize> {
    let result = (x as f64).log2();

    if result == result.floor() {
        return Some(result as usize);
    }
    None
}

#[proc_macro_derive(BitfieldSpecifier)]
pub fn bitfield_specifier(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let result = expand_bitfield_specifier(ast).unwrap_or_else(Error::into_compile_error);
    result.into()
}

fn expand_bitfield_specifier(ast: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let data = ast.data;
    let enum_ident = ast.ident;

    let mut enum_elements: Vec<&Ident> = Vec::new();
    let first_enum_ident = match data {
        syn::Data::Enum(DataEnum { ref variants, .. }) => {
            for x in variants {
                let ident = &x.ident;
                enum_elements.push(ident);
            }
            &variants[0].ident
        }
        _ => {
            unimplemented!()
        },
    };

    let discriminants_count = enum_elements.len();

    let bits_len = match logarithm_two(discriminants_count) {
        Some(number) =>  number,
        None => {
            let error = Err(syn::Error::new(
                // x.ident.span(),
                Span::call_site(),
                format!("BitfieldSpecifier expected a number of variants which is a power of 2"),
            ));
            return error;
        }
    };

    let impl_check_discrinminant_range = enum_elements.iter().map(|ident| {

        let new_ident = format_ident!("{}_check", ident);
        quote! {
            // #[allow(non_snake_case)]
            fn #new_ident() {
                let _: <<[();(#enum_ident::#ident as usize)/#discriminants_count] as MyTempTrait>::CCC as DiscriminantInRange>::PlaceHolder;
            }
        }
    });

    let get_value_inner = enum_elements.iter().map(|ident| {
        quote! {
            #enum_ident::#ident => #enum_ident::#ident,
        }
    });

    

    let impl_default_inner = enum_elements.iter().map(|ident| {
        quote! {
            if #enum_ident::#ident.discriminant() == 0 { return #enum_ident::#ident}
        }
    });

    let length_ident = format_ident!("{}_LENGTH",enum_ident);
    let output = quote! {

        #[allow(non_upper_case_globals)]
        const #length_ident:usize = #bits_len;
       
        impl #enum_ident {
            fn discriminant(&self) -> u8 {
                unsafe { *<*const _>::from(self).cast::<u8>() }
            }
        }


        impl #enum_ident {
            fn get_value(value: &<#enum_ident as Specifier>::AssocType) -> <#enum_ident as Specifier>::AssocType {
                match value {
                    #(#get_value_inner)*
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
            const BITS:usize = #bits_len;
            type AssocType = #enum_ident;
        }

        #(#impl_check_discrinminant_range)*
    };

    Ok(output)
}


#[derive(Debug)]
struct MyStruct {
    literal: Literal,
}

impl Parse for MyStruct {
    fn parse(content: ParseStream) -> Result<Self> {
        let _: Punct = content.parse()?;
        let literal: Literal = content.parse()?;
        Ok(MyStruct { literal })
    }
}
#[proc_macro_attribute]
pub fn bitfield(
    _: proc_macro::TokenStream,
    input2: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // eprintln!("input2: {:#?}", input2);


    let ast: DeriveInput = parse_macro_input!(input2);

    let result = expand_bitfield(ast).unwrap_or_else(Error::into_compile_error);
    result.into()
}

fn expand_bitfield(ast: DeriveInput) -> Result<proc_macro2::TokenStream> {
     // input2;
     let ident = ast.ident;
     let mut token_stream_vec:Vec<proc_macro2::TokenStream> = vec![];

     let data = ast.data;
     let fields = match data {
         syn::Data::Struct(syn::DataStruct { fields, .. }) => fields,
         _ => unimplemented!("here2"),
     };

    for field in fields.iter() {
        let ty = &field.ty;        
        let attrs = &field.attrs;

        for attr in attrs {
            let tokens = attr.tokens.clone().into();

            if attr.path.is_ident("bits") {
               let my_struct:MyStruct = syn::parse(tokens).unwrap();
                if let syn::Type::Path(syn::TypePath { path, .. }) = &ty {
                   let mut label_length = my_struct.literal.clone();
                   label_length.set_span(my_struct.literal.span());

                   let ty_ident = &path.segments[0].ident;
                   let fn_ident = format_ident!("{}_LENGTH_fn",ty_ident);
                   let length_ident = format_ident!("{}_LENGTH",ty_ident);

                   let x = quote!{
                    fn #fn_ident() {
                        let _: [_; #label_length] = [(); #length_ident];
                    }
                   };
                   token_stream_vec.push(x);
                }
            }
    }}

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
 
     let size_calc = fields.iter().map(|field| {
         let ty = &field.ty;
         if check_if_bool(&ty) {
             return quote! {+ 1};
         }
         quote! {+ <#ty as Specifier>::BITS}
     });
 
     let bit_length_stream = proc_macro2::TokenStream::from_iter(size_calc);
 
     let data_size = quote! {
        fn _check_data_size() {
            let _: MyType<[(); (0  #bit_length_stream) % 8]>;
        }

         #[repr(C)]
         struct #ident {
             data: [u8; (0  #bit_length_stream) / 8]
         }
     };
 
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

     let generate_length: proc_macro2::TokenStream = proc_macro2::TokenStream::from_iter(token_stream_vec);

     let output = quote! {
        #generate_length
         #(#generate_empty_enum)*
         #data_size
 
         // builder struct
         #builder_struct
 
         // builder new
         #builder_new
 
         //impl builder set & get
         #get_set
     };
 
     Ok(output.into())
}

