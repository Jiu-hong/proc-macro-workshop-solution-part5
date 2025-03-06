use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    AssocType, DeriveInput, Ident, Token,
    parse::Parse,
    parse_macro_input,
    token::{Pub, Struct},
};

#[proc_macro_attribute]
pub fn bitfield(
    _: proc_macro::TokenStream,
    input2: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    eprintln!("input2: {:#?}", input2);
    let input3_clone = input2.clone();

    let ast: DeriveInput = parse_macro_input!(input3_clone);

    let ident = ast.ident;

    let data = ast.data;
    let fields = match data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => fields,
        _ => unimplemented!(),
    };

    let builder_name = format_ident!("{}builder", ident);
    let inner_output = fields.clone().into_iter().map(|field| {
        let f_ident = field.ident.unwrap();
        let f_ty = field.ty;
        quote! {
            #f_ident:<#f_ty as Specifier>::AssocType,
        }
    });

    let builder_struct = quote! {
        struct #builder_name {
            #(#inner_output)*
        }
    };

    let set_get_inner = fields.clone().into_iter().map(|field| {
        let f_ident = field.ident.unwrap();
        let f_ty = field.ty;

        let get_func_name = format_ident!("get_{}", f_ident);
        let set_func_name = format_ident!("set_{}", f_ident);
        quote! {

                  fn #get_func_name(&self) -> <#f_ty as Specifier>::AssocType{
                      self.#f_ident
                  }
                  fn #set_func_name(&mut self,value: <#f_ty as Specifier>::AssocType) {
                      self.#f_ident=value
                  }
        }
    });
    let set_get = quote! {
                    impl #builder_name {
                        #(#set_get_inner)*
        }
    };

    let b_type_iterator = (1usize..=64).map(|number| {
        let b_type = Ident::new(&format!("B{}", number), Span::call_site());
        quote! {
            enum #b_type {}
            impl Specifier for #b_type {
                const BITS: usize = #number;
                type AssocType = u64;
            }
        }
    });
    let enum_type = proc_macro2::TokenStream::from_iter(b_type_iterator);

    let size_calc = fields.clone().into_iter().map(|field| {
        let ty = field.ty;
        quote! { + <#ty as Specifier>::BITS
        }
    });

    let new_token_stream = proc_macro2::TokenStream::from_iter(size_calc);

    let check_size = quote! {
        fn _check() {
            let _:MyType<[(); (0  #new_token_stream) % 8]>;
        }
    };

    let data_size = quote! {
        #[repr(C)]
        struct #ident {
            data: [u8; (0  #new_token_stream) / 8]
        }
    };

    let struct_inner = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        quote! {
            #ident:<#ty as Specifier>::AssocType::default(),
        }
    });
    let struct_new = quote! {
        impl #ident {
            fn new() -> #builder_name {
                #builder_name {
                    #(#struct_inner)*
                }
            }
        }
    };

    quote! {
        #builder_struct
        #struct_new
        #check_size
        #data_size
        #set_get
        #enum_type
    }
    .into()
}
