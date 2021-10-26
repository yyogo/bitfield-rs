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
                ident.span(),
            )
        })
        .collect();

    let storage = match the_enum.variants.len() {
        0..=8 => quote! {u8},
        9..=16 => quote! {u16},
        17..=32 => quote! {u32},
        33..=64 => quote! {u64},
        65..=128 => quote! {u128},
        _ => panic!("enum too large for a bit set"),
    };

    let result = quote! {
        impl bitset::BitFlag for #name {

            type Storage = #storage;

            fn bits(self) -> Self::Storage {
                (1 as #storage) << (self as usize)
            }

            fn from_index(index: usize) -> Option<Self> {
                #(const #match_constant: usize = #name::#flags as usize;)*
                match index {
                    #(#match_constant => Some(#name::#flags)),*,
                    _ => None
                }
            }
        }

        impl core::ops::BitOr for #name {
            type Output = bitset::BitSet<Self>;

            fn bitor(self, rhs: Self) -> Self::Output {
                bitset::BitSet::from(self) | bitset::BitSet::from(rhs)
            }
        }

        impl core::ops::BitOr<bitset::BitSet<#name>> for #name {
            type Output = bitset::BitSet<Self>;

            fn bitor(self, rhs:bitset::BitSet<#name>) -> Self::Output {
                rhs | bitset::BitSet::from(self)
            }
        }

        impl core::ops::Not for #name {
            type Output = bitset::BitSet<Self>;

            fn not(self) -> Self::Output {
                ! bitset::BitSet::from(self)
            }
        }

        impl ::core::convert::TryFrom<bitset::BitSet<#name>> for #name {
            type Error = bitset::GetSingleError;

            fn try_from(set: bitset::BitSet<#name>) -> Result<Self, Self::Error> {
                set.get_single()
            }
        }
    };
    result.into()
}
