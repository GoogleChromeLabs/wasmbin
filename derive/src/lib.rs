extern crate proc_macro;

use quote::{quote, ToTokens};
use synstructure::{decl_derive, Structure};

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

fn wasmbin_derive(s: Structure) -> proc_macro2::TokenStream {
    let (encode_discriminant, decode) = match s.ast().data {
        syn::Data::Enum(_) => {
            if !s.ast().attrs.iter().any(|attr| {
                attr.path.is_ident("repr")
                    && attr
                        .parse_args::<syn::Ident>()
                        .map_or(false, |ident| ident == "u8")
            }) {
                panic!("Wasmbin enums must use #[repr(u8)].");
            }

            let mut decoders = quote!();
            let mut decode_other = quote!({ return Ok(None) });

            let encode_discriminant = s.each_variant(|v| {
                v.ast()
                    .discriminant
                    .as_ref()
                    .map(|(_, discriminant)| quote!(#discriminant))
                    .or_else(|| discriminant_attr(v).map(|lit| quote!(#lit)))
                    .map_or_else(
                        || {
                            let fields = v.ast().fields;
                            assert_eq!(fields.len(), 1, "Single field is required for catch-all discriminants.");
                            let field = fields.iter().next().unwrap();
                            let construct = match &field.ident {
                                Some(ident) => quote!({ #ident: res }),
                                None => quote!((res))
                            };
                            let variant_name = v.ast().ident;
                            decode_other = quote! {
                                if let Some(res) = WasmbinDecodeWithDiscriminant::maybe_decode_with_discriminant(discriminant, r)? {
                                    Self::#variant_name #construct
                                } else #decode_other
                            };
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
                        fn maybe_decode_with_discriminant(discriminant: u8, r: &mut impl std::io::Read) -> Result<Option<Self>, DecodeError> {
                            Ok(Some(match discriminant {
                                #decoders
                                _ => #decode_other
                            }))
                        }
                    }

                    gen impl WasmbinDecode for @Self {
                        fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                            WasmbinDecodeWithDiscriminant::decode(r)
                        }
                    }
                },
            )
        }
        _ => {
            let variants = s.variants();
            assert_eq!(variants.len(), 1);
            let v = &variants[0];
            let construct = v.construct(|_, _| quote!(WasmbinDecode::decode(r)?));
            match discriminant_attr(v) {
                Some(discriminant) => (
                    quote! {
                        <u8 as WasmbinEncode>::encode(&#discriminant, w)?;
                    },
                    quote! {
                        gen impl WasmbinDecodeWithDiscriminant for @Self {
                            fn maybe_decode_with_discriminant(discriminant: u8, r: &mut impl std::io::Read) -> Result<Option<Self>, DecodeError> {
                                Ok(match discriminant {
                                    #discriminant => Some(#construct),
                                    _ => None
                                })
                            }
                        }

                        gen impl WasmbinDecode for @Self {
                            fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                                WasmbinDecodeWithDiscriminant::decode(r)
                            }
                        }
                    },
                ),
                None => (
                    quote! {},
                    quote! {
                        gen impl WasmbinDecode for @Self {
                            fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                                Ok(#construct)
                            }
                        }
                    },
                ),
            }
        }
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

fn wasmbin_countable_derive(s: Structure) -> proc_macro2::TokenStream {
    s.gen_impl(quote! {
        gen impl crate::WasmbinCountable for @Self {}
    })
}

#[proc_macro_attribute]
pub fn wasmbin_discriminants(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input: syn::DeriveInput = syn::parse(input).unwrap();
    let e = match &mut input.data {
        syn::Data::Enum(e) => e,
        _ => panic!("This attribute can only be used on enums"),
    };
    let mut seen_non_units = false;
    for v in &mut e.variants {
        match v.fields {
            syn::Fields::Unit => {}
            _ => seen_non_units = true,
        }
        if let Some((_, discriminant)) = v.discriminant.take() {
            v.attrs
                .push(syn::parse_quote!(#[wasmbin(discriminant = #discriminant)]));
        }
    }
    assert!(
        seen_non_units,
        "Attribute shouldn't be used on C-like enums"
    );
    input.into_token_stream().into()
}

decl_derive!([Wasmbin, attributes(wasmbin)] => wasmbin_derive);
decl_derive!([WasmbinCountable] => wasmbin_countable_derive);
