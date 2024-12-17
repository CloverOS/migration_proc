extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput};

#[proc_macro]
pub fn comment(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let result = if let Data::Enum(ref data_enum) = &input.data {
        let variants = data_enum.variants.clone().into_iter().map(|variant| {
            let var_name = &variant.ident;
            let docs = variant
                .attrs
                .iter()
                .find_map(|attr| {
                    if attr.path.is_ident("doc") {
                        Some(attr.tokens.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "".to_string());

            if docs.is_empty() {
                return quote! {};
            }
            let docs = docs.replace("=", "").replace(" ", "").replace("\"","");
            quote! {
                db.execute(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    format!("COMMENT ON COLUMN {}.{} IS '{}';", #name::Table.to_string(), #name::#var_name.to_string(), #docs),
                ))
                    .await?;
            }
        });

        let derive_ident_attr = Attribute {
            pound_token: Default::default(),
            style: syn::AttrStyle::Outer,
            bracket_token: Default::default(),
            path: syn::parse_str("derive").unwrap(),
            tokens: quote!((DeriveIden)).into(),
        };
        input.attrs.push(derive_ident_attr);

        let fn_name = Ident::new(
            &format!("{}_comment", name.to_string().to_ascii_lowercase()),
            input.ident.span(),
        );
        quote! {
             #input

            pub async fn #fn_name(db: &SchemaManagerConnection<'_>) -> Result<(), DbErr> {
                #(#variants)*
                Ok(())
            }
        }
    } else {
        quote! {
            compile_error!("This macro only works on enums");
        }
    };

    result.into()
}
