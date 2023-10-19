use bundle::{XmlAttribute, XmlAttributeBundle};
use proc_macro::TokenStream as TokenStream1;

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::*;
use util::flatten_result_vec;

mod bundle;
mod util;

#[proc_macro_derive(BundleAttributes, attributes(xml_attribute, xml_attribute_bundle))]
pub fn attrib_bundle(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        Data::Enum(_) => todo!("enum not supported"),
        Data::Union(_) => todo!("union not supported"),
    };

    let entries: Vec<_> = fields.iter().filter_map(XmlAttribute::new).collect();
    let checks: Vec<Expr> = match flatten_result_vec(entries) {
        Ok(it) => it
            .into_iter()
            .map(XmlAttribute::generate_write_expr)
            .collect(),
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let struct_name = &input.ident;
    let struct_generics = input.generics.clone();
    let args: Punctuated<_, token::Comma> = struct_generics
        .params
        .iter()
        .map(|it| match it {
            GenericParam::Lifetime(time) => GenericArgument::Lifetime(time.lifetime.clone()),
            GenericParam::Type(ty) => GenericArgument::Type(Type::Path(TypePath {
                qself: None,
                path: Path::from(ty.ident.clone()),
            })),
            GenericParam::Const(param) => GenericArgument::Const(Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path::from(param.ident.clone()),
            })),
        })
        .collect();
    let generic_names = AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: token::Lt::default(),
        args,
        gt_token: token::Gt::default(),
    };

    let bundles: Vec<_> = fields.iter().filter_map(XmlAttributeBundle::new).collect();
    let bundles: Vec<XmlAttributeBundle> = match flatten_result_vec(bundles) {
        Ok(it) => it,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let bundle_exprs: Vec<Expr> = bundles.iter().map(|it| it.generate_write_expr()).collect();

    let result = quote! {
        impl #struct_generics crate::io::AttributeBundle for #struct_name #generic_names {
            #[cfg(feature = "write")]
            #[allow(unused)]
            fn write_attributes<W: std::io::Write>(
                &self,
                writer: &mut W,
                settings: &crate::io::WriteSettings,
            ) -> std::io::Result<bool> {
                let mut wrote_any_attributes = false;
                #(
                    #checks
                )*
                #(
                    #bundle_exprs
                )*
                Ok(wrote_any_attributes)
            }
        }
    };

    result.into()
}
