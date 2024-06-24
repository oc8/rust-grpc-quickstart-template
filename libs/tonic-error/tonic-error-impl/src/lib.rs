use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(TonicError)]
pub fn tonic_error_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_tonic_error(&ast)
}

fn impl_tonic_error(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl<'t> TonicError<'t> for #name {}

        static CUSTOM_ERROR: &str = "x-tonic-error";

        impl From<#name> for tonic::Status {
            fn from(e: #name) -> Self {
                let type_name = to_variant_name(&e).unwrap();

                if e.is_list() {
                    let error_json = serde_json::json!({
                        "type": type_name,
                        "errors": e.errors(),
                    });

                    let mut status = tonic::Status::new(e.code(), format!("{}", type_name));

                    status.metadata_mut().insert(
                        CUSTOM_ERROR,
                        serde_json::to_string(&error_json)
                            .unwrap_or("could not serialize: {e}".to_string())
                            .parse()
                            .unwrap_or(tonic::metadata::MetadataValue::from_static("unable to create metadata value"))
                    );
                    return status;
                }

                let error_json = serde_json::json!({
                    "message": e.to_string(),
                    "type": type_name,
                });

                let mut status = tonic::Status::new(e.code(), format!("{}", type_name));

                status.metadata_mut().insert(
                    CUSTOM_ERROR,
                    serde_json::to_string(&error_json)
                        .unwrap_or("could not serialize: {e}".to_string())
                        .parse()
                        .unwrap_or(tonic::metadata::MetadataValue::from_static("unable to create metadata value"))
                );
                status
            }
        }
    };
    gen.into()
}