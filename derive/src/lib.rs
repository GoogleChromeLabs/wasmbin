// Copyright 2020 Google Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate proc_macro;

use quote::{quote, ToTokens};
use std::borrow::Cow;
use synstructure::{decl_derive, Structure, VariantInfo};

macro_rules! syn_throw {
    ($err:expr) => {
        return syn::Error::to_compile_error(&$err);
    };
}

macro_rules! syn_try {
    ($expr:expr) => {
        match $expr {
            Ok(expr) => expr,
            Err(err) => syn_throw!(err),
        }
    };
}

fn discriminant<'v>(v: &VariantInfo<'v>) -> syn::Result<Option<Cow<'v, syn::Expr>>> {
    v.ast()
        .discriminant
        .iter()
        .map(|(_, discriminant)| Ok(Cow::Borrowed(discriminant)))
        .chain(v.ast().attrs.iter().filter_map(|attr| match attr {
            syn::Attribute {
                style: syn::AttrStyle::Outer,
                path,
                ..
            } if path.is_ident("wasmbin") => {
                syn::custom_keyword!(discriminant);

                Some(
                    attr.parse_args_with(|parser: syn::parse::ParseStream| {
                        parser.parse::<discriminant>()?;
                        parser.parse::<syn::Token![=]>()?;
                        parser.parse()
                    })
                    .map(Cow::Owned),
                )
            }
            _ => None,
        }))
        .try_fold(None, |prev, discriminant| {
            let discriminant = discriminant?;
            if let Some(prev) = prev {
                let mut err = syn::Error::new_spanned(
                    discriminant,
                    "#[derive(Wasmbin)]: duplicate discriminant",
                );
                err.combine(syn::Error::new_spanned(
                    prev,
                    "#[derive(Wasmbin)]: previous discriminant here",
                ));
                return Err(err);
            }
            Ok(Some(discriminant))
        })
}

fn gen_encode_discriminant(repr: &syn::Type, discriminant: &syn::Expr) -> proc_macro2::TokenStream {
    quote!(<#repr as Encode>::encode(&#discriminant, w)?)
}

fn gen_decode(v: &VariantInfo) -> proc_macro2::TokenStream {
    v.construct(|field, index| {
        let field_name = match &field.ident {
            Some(ident) => ident.to_string(),
            None => index.to_string(),
        };
        quote!(Decode::decode(r).map_err(|err| err.in_path(PathItem::Name(#field_name)))?)
    })
}

fn parse_repr(s: &Structure) -> syn::Result<syn::Type> {
    s.ast()
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("repr"))
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &s.ast().ident,
                "Wasmbin enums must have a #[repr(type)] attribute",
            )
        })?
        .parse_args()
}

fn wasmbin_derive(s: Structure) -> proc_macro2::TokenStream {
    let (encode_discriminant, decode) = match s.ast().data {
        syn::Data::Enum(_) => {
            let repr = syn_try!(parse_repr(&s));

            let mut encode_discriminant = quote!();

            let mut decoders = quote!();
            let mut decode_other = quote!({ return Ok(None) });

            for v in s.variants() {
                let discriminant = syn_try!(discriminant(v));

                match discriminant {
                    Some(discriminant) => {
                        let pat = v.pat();

                        let encode = gen_encode_discriminant(&repr, &discriminant);
                        (quote!(#pat => #encode,)).to_tokens(&mut encode_discriminant);

                        let construct = gen_decode(v);
                        let variant_name = v.ast().ident.to_string();
                        (quote!(
                            #discriminant => (move || -> Result<_, DecodeError> {
                                Ok(#construct)
                            })()
                            .map_err(|err| err.in_path(PathItem::Variant(#variant_name)))?,
                        ))
                        .to_tokens(&mut decoders);
                    }
                    None => {
                        let fields = v.ast().fields;
                        if fields.len() != 1 {
                            syn_throw!(syn::Error::new_spanned(
                                fields,
                                "Catch-all variants without discriminant must have a single field."
                            ));
                        }
                        let field = fields.iter().next().unwrap();
                        let construct = match &field.ident {
                            Some(ident) => quote!({ #ident: res }),
                            None => quote!((res)),
                        };
                        let variant_name = v.ast().ident;
                        decode_other = quote! {
                            if let Some(res) = DecodeWithDiscriminant::maybe_decode_with_discriminant(discriminant, r)? {
                                Self::#variant_name #construct
                            } else #decode_other
                        };
                    }
                }
            }

            let name = s.ast().ident.to_string();

            (
                quote! {
                    match *self {
                        #encode_discriminant
                        _ => {}
                    }
                },
                quote! {
                    gen impl DecodeWithDiscriminant for @Self {
                        const NAME: &'static str = #name;
                        type Discriminant = #repr;

                        fn maybe_decode_with_discriminant(discriminant: #repr, r: &mut impl std::io::Read) -> Result<Option<Self>, DecodeError> {
                            Ok(Some(match discriminant {
                                #decoders
                                _ => #decode_other
                            }))
                        }
                    }

                    gen impl Decode for @Self {
                        fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                            DecodeWithDiscriminant::decode_without_discriminant(r)
                        }
                    }
                },
            )
        }
        _ => {
            let variants = s.variants();
            assert_eq!(variants.len(), 1);
            let v = &variants[0];
            let decode = gen_decode(v);
            match syn_try!(discriminant(v)) {
                Some(discriminant) => {
                    let name = s.ast().ident.to_string();
                    (
                        gen_encode_discriminant(&syn::parse_quote!(u8), &discriminant),
                        quote! {
                            gen impl DecodeWithDiscriminant for @Self {
                                const NAME: &'static str = #name;
                                type Discriminant = u8;

                                fn maybe_decode_with_discriminant(discriminant: u8, r: &mut impl std::io::Read) -> Result<Option<Self>, DecodeError> {
                                    Ok(match discriminant {
                                        #discriminant => Some(#decode),
                                        _ => None
                                    })
                                }
                            }

                            gen impl Decode for @Self {
                                fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                                    DecodeWithDiscriminant::decode_without_discriminant(r)
                                }
                            }
                        },
                    )
                }
                None => (
                    quote! {},
                    quote! {
                        gen impl Decode for @Self {
                            fn decode(r: &mut impl std::io::Read) -> Result<Self, DecodeError> {
                                Ok(#decode)
                            }
                        }
                    },
                ),
            }
        }
    };

    let encode_body = s.each(|bi| {
        quote! {
            Encode::encode(#bi, w)?
        }
    });

    s.gen_impl(quote! {
        use crate::io::{Encode, Decode, DecodeWithDiscriminant, DecodeError, PathItem};

        gen impl Encode for @Self {
            fn encode(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
                #encode_discriminant;
                match *self { #encode_body }
                Ok(())
            }
        }

        #decode
    })
}

fn wasmbin_countable_derive(s: Structure) -> proc_macro2::TokenStream {
    s.gen_impl(quote! {
        gen impl crate::builtins::WasmbinCountable for @Self {}
    })
}

fn wasmbin_visit_derive(mut s: Structure) -> proc_macro2::TokenStream {
    s.bind_with(|_| synstructure::BindStyle::Move);

    let visit_children_body = s.each(|bi| {
        quote! {
            Visit::visit_child(#bi, f)?
        }
    });

    let visit_children_mut_body = s.each(|bi| {
        quote! {
            Visit::visit_child_mut(#bi, f)?
        }
    });

    s.gen_impl(quote! {
        use crate::visit::{Visit, VisitError};

        gen impl Visit for @Self where Self: 'static {
            fn visit_children<'a, VisitT: 'static, VisitE, VisitF: FnMut(&'a VisitT) -> Result<(), VisitE>>(&'a self, f: &mut VisitF) -> Result<(), VisitError<VisitE>> {
                match self { #visit_children_body }
                Ok(())
            }

            fn visit_children_mut<VisitT: 'static, VisitE, VisitF: FnMut(&mut VisitT) -> Result<(), VisitE>>(&mut self, f: &mut VisitF) -> Result<(), VisitError<VisitE>> {
                match self { #visit_children_mut_body }
                Ok(())
            }
        }
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
        #[cfg(not(feature = "nightly"))]
        {
            if let Some((_, discriminant)) = v.discriminant.take() {
                v.attrs
                    .push(syn::parse_quote!(#[wasmbin(discriminant = #discriminant)]));
            }
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
decl_derive!([Visit] => wasmbin_visit_derive);
