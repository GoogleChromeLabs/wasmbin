#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use quote::ToTokens;

fn discriminant_attr(v: &synstructure::VariantInfo) -> Option<syn::Lit> {
    v.ast().attrs.iter().find_map(|attr| match attr {
        syn::Attribute {
            style: syn::AttrStyle::Outer,
            path,
            ..
        } if path.is_ident("wasmbin") => {
            let args = attr
                .parse_args::<syn::MetaNameValue>()
                .expect("Wrong format of wasmbin attr");
            assert!(args.path.is_ident("discriminant"));
            Some(args.lit)
        }
        _ => None,
    })
}

fn wasmbin_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let name = s.ast().ident.to_string();

    let decode_other_err = quote!(discriminant => return Err(DecodeError::UnsupportedDiscriminant {
        ty: #name,
        discriminant
    }));

    let (encode_discriminant, decode) = match s.ast().data {
        syn::Data::Enum(_) => {
            let mut seen_other = false;

            let mut decoders = quote!();
            let mut decode_other = decode_other_err;

            let encode_discriminant = s.each_variant(|v| {
                v.ast()
                    .discriminant
                    .as_ref()
                    .map(|(_, discriminant)| quote!(#discriminant))
                    .or_else(|| discriminant_attr(v).map(|lit| quote!(#lit)))
                    .map_or_else(
                        || {
                            if seen_other {
                                panic!("Maximum one variant might be without a discriminant");
                            }
                            seen_other = true;
                            let construct_other =
                                v.construct(|_, _| quote!(WasmbinDecodeWithDiscriminant::decode_with_discriminant(discriminant, r)?));
                            decode_other = quote!(discriminant => #construct_other);
                            quote!(Ok(()))
                        },
                        |discriminant| {
                            let construct = v.construct(|_, _| quote!(WasmbinDecode::decode(r)?));
                            (quote!(#discriminant => #construct,)).to_tokens(&mut decoders);
                            quote!(<u8 as WasmbinEncode>::encode(&#discriminant, w))
                        },
                    )
            });

            (
                quote! {
                    match *self {
                        #encode_discriminant
                    }?;
                },
                quote! {
                    gen impl WasmbinDecodeWithDiscriminant for @Self {
                        fn decode_with_discriminant(discriminant: u8, r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                            Ok(match discriminant {
                                #decoders
                                #decode_other
                            })
                        }
                    }
                }
            )
        }
        _ => {
            let variants = s.variants();
            assert_eq!(variants.len(), 1);
            let v = &variants[0];
            let construct = v.construct(|_, _| quote!(WasmbinDecode::decode(r)?));
            match discriminant_attr(v) {
                Some(discriminant) => (quote! {
                    <u8 as WasmbinEncode>::encode(&#discriminant, w)?;
                }, quote! {
                    gen impl WasmbinDecodeWithDiscriminant for @Self {
                        fn decode_with_discriminant(discriminant: u8, r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                            Ok(match discriminant {
                                #discriminant => #construct,
                                #decode_other_err
                            })
                        }
                    }
                }),
                None => (quote!{}, quote! {
                    gen impl WasmbinDecode for @Self {
                        fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                            Ok(#construct)
                        }
                    }
                })
            }
        },
    };

    let encode_body = s.each(|bi| {
        quote! {
            WasmbinEncode::encode(#bi, w)?
        }
    });

    s.gen_impl(quote! {
        use crate::{WasmbinEncode, WasmbinDecode, WasmbinDecodeWithDiscriminant, DecodeError};

        gen impl WasmbinEncode for @Self {
            fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                #encode_discriminant
                match *self { #encode_body }
                Ok(())
            }
        }

        #decode
    })
}

fn wasmbin_countable_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    s.gen_impl(quote! {
        gen impl crate::WasmbinCountable for @Self {}
    })
}

decl_derive!([Wasmbin, attributes(wasmbin)] => wasmbin_derive);
decl_derive!([WasmbinCountable] => wasmbin_countable_derive);
