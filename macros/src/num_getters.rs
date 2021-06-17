//use std::string::ToString;
use proc_macro2::{
    Group as PmGroup, Ident as PmIdent, Literal as PmLiteral, Span, TokenStream, TokenTree,
};
use quote::{quote, TokenStreamExt as _};
use syn::{Expr, Ident, Stmt, Token};

#[derive(Clone, Copy)]
pub enum Endidness {
    Big,
    Little,
    Native,
}

impl Endidness {
    fn short(&self) -> &str {
        match self {
            Self::Big => "be",
            Self::Little => "le",
            Self::Native => "ne",
        }
    }
}

struct NumberInfo {
    ident: &'static str,
    width: u8,
}

impl NumberInfo {
    fn new(ident: &'static str, width: u8) -> Self {
        Self { ident, width }
    }

    fn width(&self) -> TokenTree {
        TokenTree::Literal(PmLiteral::u8_unsuffixed(self.width))
    }

    fn ident(&self) -> TokenTree {
        TokenTree::Ident(PmIdent::new(self.ident, Span::call_site()))
    }

    fn apply_to_stream(&self, stream: TokenStream, endidness: Endidness) -> TokenStream {
        stream
            .into_iter()
            .map(|token| self.apply_to_tree(token, endidness))
            .collect()
    }

    fn apply_to_tree(&self, tree: TokenTree, endidness: Endidness) -> TokenTree {
        match tree {
            TokenTree::Group(group) => TokenTree::Group(PmGroup::new(
                group.delimiter(),
                self.apply_to_stream(group.stream(), endidness),
            )),
            TokenTree::Ident(ident) => {
                let ident_str = ident.to_string();
                if ident_str == "_numname_" {
                    self.ident()
                } else if ident_str == "_numwidth_" {
                    self.width()
                } else {
                    TokenTree::Ident(PmIdent::new(
                        &ident_str
                            .replace("numend", endidness.short())
                            .replace("numname", self.ident)
                            .replace("numwidth", &self.width.to_string()),
                        ident.span(),
                    ))
                }
            }
            _ => tree,
        }
    }
}

lazy_static! {
    static ref I8_NUM_INFO: NumberInfo = NumberInfo::new("i8", 1);
    static ref NUMBERS: Vec<NumberInfo> = vec![
        NumberInfo::new("u16", 2),
        NumberInfo::new("i16", 2),
        NumberInfo::new("u32", 4),
        NumberInfo::new("i32", 4),
        NumberInfo::new("u64", 8),
        NumberInfo::new("i64", 8),
        NumberInfo::new("u128", 16),
        NumberInfo::new("i128", 16),
    ];
}

pub fn impl_next_methods(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let in_stream = TokenStream::from(stream);
    let mut out = generate_next_impl(&I8_NUM_INFO, Endidness::Native, &in_stream);
    for num_info in NUMBERS.iter() {
        out.extend(generate_next_impl(num_info, Endidness::Little, &in_stream));
        out.extend(generate_next_impl(num_info, Endidness::Big, &in_stream));
    }
    out.into()
}

fn generate_next_impl(
    num_info: &NumberInfo,
    endidness: Endidness,
    stream: &TokenStream,
) -> TokenStream {
    let impl_meth_name = Ident::new(
        &format!("next_{}_{}", num_info.ident, endidness.short()),
        Span::call_site(),
    );
    let body = num_info.apply_to_stream(stream.clone(), endidness);
    let rtype = num_info.ident();
    quote! {
        fn #impl_meth_name(&mut self) -> binreader::Result<#rtype> {
            #body
        }
    }
}

pub fn impl_at_methods(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let in_stream = TokenStream::from(stream);
    let mut out = generate_at_impl(&I8_NUM_INFO, Endidness::Native, &in_stream);
    for num_info in NUMBERS.iter() {
        out.extend(generate_at_impl(num_info, Endidness::Little, &in_stream));
        out.extend(generate_at_impl(num_info, Endidness::Big, &in_stream));
    }
    out.into()
}

fn generate_at_impl(
    num_info: &NumberInfo,
    endidness: Endidness,
    stream: &TokenStream,
) -> TokenStream {
    let impl_meth_name = Ident::new(
        &format!("{}_{}_at", num_info.ident, endidness.short()),
        Span::call_site(),
    );
    let body = num_info.apply_to_stream(stream.clone(), endidness);
    let rtype = num_info.ident();
    quote! {
        fn #impl_meth_name(&self, offset: usize) -> binreader::Result<#rtype> {
            #body
        }
    }
}

pub fn make_number_methods(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let in_stream = TokenStream::from(stream);
    let mut out = I8_NUM_INFO.apply_to_stream(in_stream.clone(), Endidness::Native);
    for num_info in NUMBERS.iter() {
        out.extend(num_info.apply_to_stream(in_stream.clone(), Endidness::Big));
        out.extend(num_info.apply_to_stream(in_stream.clone(), Endidness::Little));
    }
    out.into()
}
