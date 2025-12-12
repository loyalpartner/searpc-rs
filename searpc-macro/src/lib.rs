//! # searpc-macro: Type-safe RPC client generation
//!
//! Procedural macro for generating type-safe RPC clients from trait definitions.
//!
//! ## Design Philosophy
//!
//! Following Linus Torvalds' "good taste" principles:
//! - **Data structures first**: Strong typing eliminates runtime errors
//! - **Zero special cases**: Uniform code generation for all types
//! - **Type safety**: Compiler catches errors, not runtime panics
//!
//! ## Example
//!
//! ```rust,ignore
//! use searpc_macro::rpc;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Repo {
//!     id: String,
//!     name: String,
//! }
//!
//! #[rpc]
//! trait SeafileRpc {
//!     #[rpc(name = "get_version")]
//!     fn get_version(&self) -> Result<String>;
//!
//!     #[rpc(name = "list_repos")]
//!     fn list_repos(&self, offset: i32, limit: i32) -> Result<Vec<Repo>>;
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{
    parse_macro_input, DeriveInput, FnArg, ItemTrait, PatType, ReturnType, TraitItem, TraitItemFn,
    Type,
};

/// Main procedural macro for generating RPC client implementations
///
/// # Usage
///
/// ## With prefix (recommended)
///
/// ```rust,ignore
/// #[rpc(prefix = "seafile")]
/// trait SeafileRpc {
///     fn get_version(&mut self) -> Result<String>;
///     // Calls: seafile_get_version
///
///     fn list_repos(&mut self, offset: i32) -> Result<Vec<Repo>>;
///     // Calls: seafile_list_repos
///
///     #[rpc(name = "get_commit")]
///     fn get_specific_commit(&mut self, id: &str) -> Result<Commit>;
///     // Calls: get_commit (override)
/// }
/// ```
///
/// ## With service and prefix
///
/// ```rust,ignore
/// #[rpc(service = "seafile-rpcserver", prefix = "seafile")]
/// trait SeafileRpc {
///     fn get_version(&mut self) -> Result<String>;
/// }
/// ```
///
/// ## Manual naming (for compatibility)
///
/// ```rust,ignore
/// #[rpc]
/// trait MyRpc {
///     #[rpc(name = "remote_function_name")]
///     fn local_name(&mut self, arg: Type) -> Result<ReturnType>;
/// }
/// ```
///
/// ## Expand struct arguments
///
/// Use `#[derive(ExpandArgs)]` and `#[rpc(expand)]` to pass a struct
/// whose fields are expanded into individual RPC arguments:
///
/// ```rust,ignore
/// #[derive(ExpandArgs)]
/// struct CreateRepoRequest {
///     name: String,
///     desc: String,
///     owner: String,
/// }
///
/// #[rpc(prefix = "seafile")]
/// trait SeafileRpc {
///     #[rpc(expand)]
///     fn create_repo(&mut self, req: CreateRepoRequest) -> Result<String>;
///     // Calls: seafile_create_repo(name, desc, owner)
/// }
/// ```
#[proc_macro_attribute]
pub fn rpc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemTrait);

    match generate_rpc_impl(&input, attr.into()) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

/// Configuration from trait-level #[rpc(...)] attribute
struct RpcConfig {
    service: Option<String>,
    prefix: Option<String>,
}

/// Parse trait-level #[rpc(...)] attributes
fn parse_rpc_config(attrs: proc_macro2::TokenStream) -> syn::Result<RpcConfig> {
    let mut config = RpcConfig {
        service: None,
        prefix: None,
    };

    if attrs.is_empty() {
        return Ok(config);
    }

    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("service") {
            config.service = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("prefix") {
            config.prefix = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else {
            Err(meta.error("unsupported attribute"))
        }
    });

    parser.parse2(attrs)?;
    Ok(config)
}

/// Generate the RPC implementation for a trait
fn generate_rpc_impl(
    trait_def: &ItemTrait,
    attrs: proc_macro2::TokenStream,
) -> syn::Result<TokenStream> {
    let trait_name = &trait_def.ident;
    let trait_generics = &trait_def.generics;
    let trait_vis = &trait_def.vis;
    let trait_attrs = &trait_def.attrs;

    // Parse trait-level configuration
    let config = parse_rpc_config(attrs)?;

    // Collect trait methods (keep original signatures for trait definition)
    let trait_methods: Vec<_> = trait_def
        .items
        .iter()
        .filter_map(|item| {
            if let TraitItem::Fn(method) = item {
                Some(method)
            } else {
                None
            }
        })
        .collect();

    // Generate implementations for each method
    let mut method_impls = Vec::new();
    for method in &trait_methods {
        let method_impl = generate_method_impl(method, &config)?;
        method_impls.push(method_impl);
    }

    // Rebuild trait definition with original methods
    let trait_methods_for_def: Vec<_> = trait_methods
        .iter()
        .map(|method| {
            let sig = &method.sig;
            let attrs: Vec<_> = method
                .attrs
                .iter()
                .filter(|attr| !attr.path().is_ident("rpc"))
                .collect();

            quote! {
                #(#attrs)*
                #sig;
            }
        })
        .collect();

    // Generate the complete output
    let expanded = quote! {
        #(#trait_attrs)*
        #trait_vis trait #trait_name #trait_generics {
            #(#trait_methods_for_def)*
        }

        impl<T: ::searpc::Transport> #trait_name #trait_generics for ::searpc::SearpcClient<T> {
            #(#method_impls)*
        }
    };

    Ok(expanded.into())
}

/// Generate implementation for a single RPC method
fn generate_method_impl(
    method: &TraitItemFn,
    config: &RpcConfig,
) -> syn::Result<proc_macro2::TokenStream> {
    // Determine RPC function name and method config
    let (rpc_name, method_config) = determine_rpc_config(method, config)?;

    // Parse parameters (skip self)
    let args = extract_args(&method.sig.inputs)?;

    // Determine return type and generate appropriate call
    let return_type = match &method.sig.output {
        ReturnType::Type(_, ty) => ty.as_ref(),
        _ => {
            return Err(syn::Error::new_spanned(
                &method.sig,
                "RPC methods must return Result<T>",
            ))
        }
    };

    let (call_expr, deserialize_expr) =
        generate_call_expression(return_type, &rpc_name, &args, method_config.expand)?;

    // Build the method implementation
    // Filter out #[rpc(...)] attributes to avoid duplication
    let sig = &method.sig;
    let filtered_attrs: Vec<_> = method
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("rpc"))
        .collect();

    Ok(quote! {
        #(#filtered_attrs)*
        #sig {
            #call_expr
            #deserialize_expr
        }
    })
}

/// Determine the RPC function name and method config
///
/// Priority for name:
/// 1. Method-level #[rpc(name = "...")] if present
/// 2. prefix + "_" + method_name if prefix configured
/// 3. method_name as-is
fn determine_rpc_config(
    method: &TraitItemFn,
    config: &RpcConfig,
) -> syn::Result<(String, MethodRpcConfig)> {
    // Get method-level config
    let method_config = try_extract_method_config(&method.attrs)?;

    // Determine RPC name
    let rpc_name = if let Some(name) = &method_config.name {
        name.clone()
    } else {
        let method_name = method.sig.ident.to_string();
        if let Some(prefix) = &config.prefix {
            format!("{}_{}", prefix, method_name)
        } else {
            method_name
        }
    };

    Ok((rpc_name, method_config))
}

/// Method-level RPC configuration
struct MethodRpcConfig {
    name: Option<String>,
    expand: bool,
}

/// Try to extract RPC config from method-level #[rpc(...)]
fn try_extract_method_config(attrs: &[syn::Attribute]) -> syn::Result<MethodRpcConfig> {
    let mut config = MethodRpcConfig {
        name: None,
        expand: false,
    };

    for attr in attrs {
        if attr.path().is_ident("rpc") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    config.name = Some(lit.value());
                    Ok(())
                } else if meta.path.is_ident("expand") {
                    config.expand = true;
                    Ok(())
                } else {
                    Err(meta.error("expected `name` or `expand`"))
                }
            })?;
        }
    }
    Ok(config)
}

/// Extract function arguments (excluding self)
fn extract_args(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> syn::Result<Vec<ArgInfo>> {
    let mut args = Vec::new();

    for input in inputs {
        match input {
            FnArg::Receiver(_) => continue, // Skip self
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let arg_name = quote!(#pat).to_string();
                args.push(ArgInfo {
                    name: arg_name,
                    ty: ty.as_ref().clone(),
                });
            }
        }
    }

    Ok(args)
}

struct ArgInfo {
    name: String,
    ty: Type,
}

/// Generate the RPC call expression based on return type
fn generate_call_expression(
    return_type: &Type,
    rpc_name: &str,
    args: &[ArgInfo],
    expand: bool,
) -> syn::Result<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    // Build args vector
    let args_vec = if expand {
        // expand mode: expect exactly one struct argument, expand its fields
        if args.len() != 1 {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "expand mode requires exactly one struct argument",
            ));
        }
        let arg = &args[0];
        let arg_ident = syn::Ident::new(&arg.name, proc_macro2::Span::call_site());
        let ty = &arg.ty;

        // Generate code that expands the struct fields at runtime
        // The struct must implement ExpandArgs trait
        quote! {
            let args = <#ty as ::searpc::ExpandArgs>::expand_args(#arg_ident);
        }
    } else {
        // Normal mode: convert each argument
        let arg_conversions = args.iter().map(|arg| {
            let arg_ident = syn::Ident::new(&arg.name, proc_macro2::Span::call_site());
            let ty = &arg.ty;

            // Type-based conversion to Arg
            quote! {
                {
                    let val = #arg_ident;
                    <#ty as ::searpc::IntoArg>::into_arg(val)
                }
            }
        });

        quote! {
            let args = vec![#(#arg_conversions),*];
        }
    };

    // Parse Result<T> to extract T
    let inner_type = extract_result_type(return_type)?;

    // Generate appropriate call based on type
    let (call_method, deserialize) = match_return_type(inner_type)?;

    let call_expr = quote! {
        #args_vec
        let result = self.#call_method(#rpc_name, args)?;
    };

    Ok((call_expr, deserialize))
}

/// Extract T from Result<T>
fn extract_result_type(ty: &Type) -> syn::Result<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return Ok(inner);
                    }
                }
            }
        }
    }
    Err(syn::Error::new_spanned(ty, "Expected Result<T>"))
}

/// Match return type and generate appropriate call method
fn match_return_type(
    ty: &Type,
) -> syn::Result<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    // Check for primitive types
    if is_type(ty, "String") {
        return Ok((quote!(call_string), quote!(Ok(result))));
    }
    if is_type(ty, "i32") {
        return Ok((quote!(call_int), quote!(Ok(result))));
    }
    if is_type(ty, "i64") {
        return Ok((quote!(call_int64), quote!(Ok(result))));
    }
    if is_type(ty, "bool") {
        return Ok((quote!(call_int), quote!(Ok(result != 0))));
    }

    // Check for Option<T> and Vec<T>
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            // Check for Option<T>
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(_inner)) = args.args.first() {
                        // Option<T> - use call_object and return None on null
                        return Ok((
                            quote!(call_object),
                            quote! {
                                if result.is_null() {
                                    Ok(None)
                                } else {
                                    ::serde_json::from_value(result)
                                        .map(|v| Some(v))
                                        .map_err(|e| ::searpc::SearpcError::TypeError(
                                            format!("Failed to deserialize Option: {}", e)
                                        ))
                                }
                            },
                        ));
                    }
                }
            }

            // Check for Vec<T>
            if segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        // Vec<T> - use call_objlist and deserialize
                        return Ok((
                            quote!(call_objlist),
                            quote! {
                                result.into_iter()
                                    .map(|v| ::serde_json::from_value(v)
                                        .map_err(|e| ::searpc::SearpcError::TypeError(
                                            format!("Failed to deserialize Vec element: {}", e)
                                        )))
                                    .collect::<::searpc::Result<Vec<#inner>>>()
                            },
                        ));
                    }
                }
            }
        }
    }

    // Default: single object deserialization
    Ok((
        quote!(call_object),
        quote! {
            ::serde_json::from_value(result)
                .map_err(|e| ::searpc::SearpcError::TypeError(
                    format!("Failed to deserialize: {}", e)
                ))
        },
    ))
}

/// Check if a type matches a specific name
fn is_type(ty: &Type, name: &str) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == name;
        }
    }
    false
}

/// Derive macro for implementing ExpandArgs trait on structs
///
/// This macro automatically implements the `ExpandArgs` trait for a struct,
/// allowing its fields to be expanded into RPC arguments.
///
/// # Example
///
/// ```rust,ignore
/// use searpc_macro::ExpandArgs;
///
/// #[derive(ExpandArgs)]
/// struct CreateRepoRequest {
///     name: String,
///     desc: String,
///     owner: String,
/// }
///
/// #[rpc]
/// trait MyRpc {
///     #[rpc(expand)]
///     fn create_repo(&mut self, req: CreateRepoRequest) -> Result<String>;
/// }
/// ```
#[proc_macro_derive(ExpandArgs)]
pub fn derive_expand_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_expand_args_impl(&input) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

fn generate_expand_args_impl(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract struct fields
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "ExpandArgs only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ExpandArgs can only be derived for structs",
            ))
        }
    };

    // Generate field expansions
    let field_expansions = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        quote! {
            <#field_ty as ::searpc::IntoArg>::into_arg(self.#field_name)
        }
    });

    let expanded = quote! {
        impl #impl_generics ::searpc::ExpandArgs for #name #ty_generics #where_clause {
            fn expand_args(self) -> Vec<::searpc::Arg> {
                vec![
                    #(#field_expansions),*
                ]
            }
        }
    };

    Ok(expanded.into())
}
