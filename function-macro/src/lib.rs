use proc_macro::{TokenStream, Span};
use syn::{self, ItemFn, Ident, FnArg}; 
use quote::quote; 

fn make_ascii_titlecase(s: &mut str) {
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
}

#[proc_macro_attribute]
pub fn function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: ItemFn = syn::parse(item).unwrap();
    create_excel_function(ast)
}

fn create_excel_function(ast: ItemFn) -> TokenStream {
    let function_name = ast.sig.ident.clone(); 
    let mut struct_name: String = ast.sig.ident.to_string(); 
    make_ascii_titlecase(&mut struct_name); 
    let struct_name_ident: Ident = Ident::new(&struct_name, Span::call_site().into()); 
    let fn_args = ast.sig.inputs
        .iter()
        .filter(|fnarg| { matches!(fnarg, FnArg::Typed(_)) })
        .collect::<Vec<&FnArg>>(); 

    let struct_fields = fn_args.clone().into_iter().map(|fnarg| {
        quote! {
            pub #fnarg 
        }
    });  

    let field_declarations = fn_args.clone().into_iter().map(|fnarg| {
        if let FnArg::Typed(pat_type) = fnarg {
            let arg_name = *pat_type.pat.clone(); 
            if let syn::Pat::Ident(pat_ident) = arg_name {
                if pat_ident.ident.to_string() == "args" {
                    quote! {
                        let args = v; 
                    }
                } else {
                    quote! {
                        let #fnarg = Value::from(v.pop().unwrap()); 
                    }
                }
            } else {
                quote! {}
            }
       } else {
            quote! {}
       }
    }); 

    let self_arg_declarations = fn_args.clone().into_iter().map(|fnarg| {
        if let FnArg::Typed(pat_type) = fnarg {
            let arg_name = *pat_type.pat.clone(); 
            quote! {
                self.#arg_name
            }
        } else {
            quote! { }
        }
    });

    let arg_declarations = fn_args.clone().into_iter().map(|fnarg| {
        if let FnArg::Typed(pat_type) = fnarg {
            let arg_name = *pat_type.pat.clone(); 
            quote! {
                #arg_name
            }
        } else {
            quote! { }
        }
    });

    quote! {
        pub struct #struct_name_ident {
            #(#struct_fields),* 
        }

        impl #struct_name_ident {
            #ast 
        }

        impl Function for #struct_name_ident {
            fn evaluate(self) -> Value {
                Self::#function_name(#(#self_arg_declarations),*)
            }
        }

        impl From<Vec<Value>> for #struct_name_ident { 
            fn from(mut v: Vec<Value>) -> #struct_name_ident {
                #(#field_declarations)*; 
                #struct_name_ident {#(#arg_declarations),*}
            }
        }
    }.into()
}

