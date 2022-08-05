use quote::quote;
use syn::{DeriveInput, Ident, Fields, Visibility, Type}; 
use proc_macro::TokenStream;

#[proc_macro_derive(FromArgs)]
pub fn from_args(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	impl_from_args(&ast)
}

fn impl_from_args(ast: &DeriveInput) -> TokenStream {
	let name = ast.ident.clone();
	let fields = filter_fields(match ast.data {
		syn::Data::Struct(ref s) => &s.fields,
		_ => panic!("FieldNames can only be derived for structs!")
	}); 
	let field_declarations = fields.iter().map(|(_, ident, ty)| {
		quote! {
			let #ident: #ty = Value::from(*v.pop().unwrap()); 
		}	
	});
	let field_names = fields.iter().map(|(_, ident,_)| { ident.clone() }); 
    quote! {
        impl From<Vec<Box<Expr>>> for #name {
            fn from(mut v: Vec<Box<Expr>>) -> #name {
                #(#field_declarations)*;
                #name { #(#field_names),* }
            }
        }
    }.into()
}

fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident, Type)> {
    fields
        .iter()
        .filter_map(|field| {
            if field.ident.is_some()
            {
                let field_vis = field.vis.clone();
                let field_ident = field.ident.as_ref().unwrap().clone();
				let field_ty = field.ty.clone(); 
                Some((field_vis, field_ident, field_ty))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
