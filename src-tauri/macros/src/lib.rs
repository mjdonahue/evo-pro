use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemImpl, Path, PathArguments, PathSegment, Type, TypePath, parse_macro_input};

/// A procedural attribute macro to automatically implement the `Askable` trait
/// and call the `signed_impl` macro.
///
/// This macro inspects an `impl Message<Actor>` block, extracts the necessary types,
/// and generates two pieces of code:
/// 1. An `impl Askable<Actor>` block with the correctly unwrapped reply type.
/// 2. A call to `signed_impl!(Actor, MessageType)`.
#[proc_macro_attribute]
pub fn askable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree representing the `impl` block.
    let item_impl = parse_macro_input!(item as ItemImpl);

    // --- Extract necessary types from the `impl` block ---

    // 1. Get the message type `T` from `impl Message<A> for T`.
    let self_ty = &item_impl.self_ty;

    // 2. Get the actor type `A` from `impl Message<A> for T`.
    let trait_path = if let Some((_, path, _)) = &item_impl.trait_ {
        path
    } else {
        panic!("#[askable] can only be used on an `impl` block for a trait.");
    };

    let last_segment = trait_path
        .segments
        .last()
        .expect("Trait path cannot be empty.");

    if last_segment.ident != "Message" {
        panic!("#[askable] attribute must be used on an `impl Message<...>` block.");
    }

    let message_ty = if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
        if let Some(actor_arg) = args.args.first() {
            actor_arg
        } else {
            panic!("#[askable] `Message` trait is missing its actor generic argument.");
        }
    } else {
        panic!(
            "#[askable] `Message` trait is missing its actor generic argument in angle brackets, e.g., `Message<MyActor>`."
        );
    };

    // 3. Find the `type Reply = ...;` associated type within the `impl` block.
    let reply_type = item_impl
        .items
        .iter()
        .find_map(|item| {
            if let syn::ImplItem::Type(ty) = item {
                if ty.ident == "Reply" {
                    Some(&ty.ty)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Missing `type Reply = ...;` in the `impl Message` block.");

    // --- Determine the `ActualReply` by unwrapping wrapper types ---

    let actual_reply_type = unwrap_reply_type(reply_type);

    // --- Generate the expanded code ---

    let expanded = quote! {
        // First, output the original `impl Message` block unmodified.
        #item_impl

        // Second, generate and append our new `impl Askable` block.
        impl crate::actors::Askable<#message_ty> for #self_ty {
            type ActualReply = #actual_reply_type;
        }

        // Third, generate the call to the signed_impl macro, killing the second bird.
        // We assume `signed_impl` is accessible via this path.
        #[automatically_derived]
        crate::signed_impl!(#message_ty, #self_ty);
    };

    // Return the generated code as a TokenStream.
    TokenStream::from(expanded)
}

/// Helper function to inspect a `Type` and unwrap it if it is a
/// `DelegatedReply<T>` or `ForwardedReply<T>`.
fn unwrap_reply_type(ty: &Type) -> &Type {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(PathSegment { ident, arguments }) = segments.last() {
            if ident == "DelegatedReply" || ident == "ForwardedReply" {
                if let PathArguments::AngleBracketed(args) = arguments {
                    // If it's a wrapper, return the inner type `T`.
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return inner_ty;
                    }
                }
            }
        }
    }
    // If it's not a wrapper, return the type as is.
    ty
}
