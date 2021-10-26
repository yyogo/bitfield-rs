use proc_macro2::Span;
use syn::parse_macro_input;

use quote::quote;
use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(BitFlag)]
pub fn derive_bit_flag(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let the_enum = match input.data {
        Data::Enum(e) => e,
        _ => panic!("derive(BitSet) may only be used on enums"),
    };

    let flags: Vec<_> = the_enum
        .variants
        .iter()
        .map(|var| match var.fields {
            Fields::Named(_) | Fields::Unnamed(_) => {
                panic!("derive(BitSet) may only be used on unit enums")
            }
            Fields::Unit => var.ident.clone(),
        })
        .collect();

    let match_constant: Vec<_> = flags
        .iter()
        .map(|ident| {
            syn::Ident::new(
                &format!("_const_{}_{}", name, ident).to_uppercase(),
                Span::mixed_site(),
            )
        })
        .collect();

    let storage = match the_enum.variants.len() {
        0..=8 => quote! {::core::primitive::u8},
        9..=16 => quote! {::core::primitive::u16},
        17..=32 => quote! {::core::primitive::u32},
        33..=64 => quote! {::core::primitive::u64},
        65..=128 => quote! {::core::primitive::u128},
        _ => panic!("enum too large for a bit set"),
    };

    let result = quote! {
        impl ::bitset::BitFlag for #name {

            type Storage = #storage;

            #[inline(always)]
            fn bits(self) -> Self::Storage {
                (1 as #storage) << (self as ::core::primitive::usize)
            }

            #[inline(always)]
            fn from_index(index: usize) -> ::core::option::Option<Self> {
                #(const #match_constant: ::core::primitive::usize = #name::#flags as ::core::primitive::usize;)*
                match index {
                    #(#match_constant => ::core::option::Option::Some(#name::#flags)),*,
                    _ => ::core::option::Option::None
                }
            }
        }

        impl ::core::ops::BitOr for #name {
            type Output = ::bitset::BitSet<Self>;

            fn bitor(self, rhs: Self) -> Self::Output {
                ::bitset::BitSet::single(self) | ::bitset::BitSet::single(rhs)
            }
        }

        impl ::core::ops::BitOr<::bitset::BitSet<#name>> for #name {
            type Output = ::bitset::BitSet<Self>;

            fn bitor(self, rhs: ::bitset::BitSet<#name>) -> Self::Output {
                rhs | ::bitset::BitSet::single(self)
            }
        }

        impl ::core::ops::Not for #name {
            type Output = ::bitset::BitSet<Self>;

            fn not(self) -> Self::Output {
                !::bitset::BitSet::single(self)
            }
        }

        impl ::core::convert::TryFrom<::bitset::BitSet<#name>> for #name {
            type Error = ::bitset::GetSingleError;

            fn try_from(set: ::bitset::BitSet<#name>) -> ::core::result::Result<Self, Self::Error> {
                set.get_single()
            }
        }
    };
    result.into()
}
