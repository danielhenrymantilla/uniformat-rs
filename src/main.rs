#![feature(core_intrinsics, specialization)]
#![allow(unused)]
#![deny(unused_must_use)]
// #![recursion_limit = "256"]

#[macro_use]
extern crate fstrings;

use ::std::{*,
    borrow::{Cow},
    iter::FromIterator,
    ops::Not,
    result::Result,
};
use ::itertools::Itertools;
use ::proc_macro2::{
    Span,
    TokenStream,
};
use ::quote::{
    quote,
    ToTokens,
};
use ::syn::{*,
    parse::Parse,
};

use ::std::{};

mod comments;

pub use self::context::Context;
mod context;

trait PrettyPrint {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    ;
}

fn main ()
{
    fn main ()
        -> Result<
            (),
            Cow<'static, str>,
        >
    {Ok({
        let ref filename =
            ::std::env::args()
                .nth(1)
                .expect("Please provide a filename")
        ;
        let contents =
            fs::read_to_string(filename)
                .map_err(|err|
                    format_f!("Failed to read `{filename}` error: {err}")
                )?
        ;
        let token_stream: TokenStream =
            contents
                .parse()
                .map_err(|err| format_f!("Syntax error: {err:?}"))?
        ;
        println_f!("{token_stream}");
        let source_file: File =
            parse2(token_stream)
                .map_err(|err| format_f!("Invalid source file: {err}"))?
        ;
        let ref cx = Context::new();
        println_f!("{source_file:?}\n\n{}", source_file.pp(cx));
        dbg!(comments::find_comments(&contents));
    })}

    if let Err(msg) = main() {
        eprintln_f!("{msg}");
        ::std::process::exit(1);
    }
}

impl<T : fmt::Debug> PrettyPrint for T {
    default
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        let typename: &str = unsafe {
            ::std::intrinsics::type_name::<T>()
        };
        f!("<TODO: {typename}.pp(cx)>", )
    }
}

impl PrettyPrint for File {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        let &Self {
            ref shebang,
            ref attrs,
            ref items,
        } = self;
        shebang
            .as_ref()
            .map(String::clone)
            .into_iter()
            .chain(Option::into_iter(
                if attrs.is_empty().not() {
                    Some(
                        attrs
                            .iter()
                            .map(|x| x.pp(cx))
                            .join("\n\n")
                    )
                } else {
                    None
                }
            ))
            .chain(Option::into_iter(
                if items.is_empty().not() {
                    Some(
                        items
                            .iter()
                            .map(|x| x.pp(cx))
                            .join("\n\n")
                    )
                } else {
                    None
                }
            ))
            .join("\n\n")
    }
}

impl PrettyPrint for Attribute {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        let &Self {
            pound_token: _,
            ref style,
            bracket_token: _,
            ref path,
            ref tokens,
        } = self;
        format!(
            "{indent}#{pound}[{inner}]",
            indent = cx.indent(),
            pound = if let &AttrStyle::Inner(_) = style {
                "!"
            } else {
                ""
            },
            inner = if let Ok(meta) = self.parse_meta() {
                meta.pp(cx)
            } else {
                let path = path.pp(cx);
                f!("{path} {tokens}")
            },
        )
    }
}

impl PrettyPrint for Path {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        f!(
            "{leading_colon}{segments}",
            leading_colon = self.leading_colon.map(|_| "::").unwrap_or(""),
            segments =
                self.segments
                    .iter()
                    .map(|segment| {
                        format!(
                            "{ident}{arguments}",
                            ident = segment.ident,
                            arguments = segment.arguments.pp(cx),
                        )
                    })
                    .format("::")
            ,
        )
    }
}

impl PrettyPrint for PathArguments {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        use PathArguments::*;
        match self {
            | &None => "".into(),
            | &AngleBracketed(ref angle_bracketed) => {
                unimplemented!("TODO: AngleBracketed")
            },
            | &Parenthesized(ParenthesizedGenericArguments {
                paren_token: _,
                ref inputs,
                ref output,
            }) => {
                format!("({args}){ret_ty}",
                    ret_ty = match output {
                        | &ReturnType::Default => "".into(),
                        | &ReturnType::Type(_, ref ty) => ty.pp(cx),
                    },
                    args =
                        inputs
                            .iter()
                            .map(|x| x.pp(cx))
                            .format(", ")
                    ,
                )
            },
        }
    }
}

impl PrettyPrint for Meta {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        match self {
            | &Meta::Path(ref path) => path.pp(cx),
            | &Meta::NameValue(ref name_value) => {
                let path = name_value.path.pp(cx);
                let lit = name_value.lit.pp(cx);
                f!("{path} = {lit}")
            },
            | &Meta::List(ref list) => {
                let path = list.path.pp(cx);
                f!("{path}({nested})", nested = {
                    list.nested
                        .iter()
                        .map(|nested_meta| match nested_meta {
                            | &NestedMeta::Meta(ref meta) => meta.pp(cx),
                            | &NestedMeta::Lit(ref lit) => lit.pp(cx),
                        })
                        .format(", ")
                })
            },
        }
    }
}

impl PrettyPrint for Lit {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        self.clone()
            .into_token_stream()
            .to_string()
    }
}

impl PrettyPrint for Item {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        match self {
            | &Item::Const(ref itemconst) => itemconst.pp(cx),
            | &Item::Enum(ref itemenum) => itemenum.pp(cx),
            | &Item::ExternCrate(ref itemexterncrate) => itemexterncrate.pp(cx),
            | &Item::Fn(ref itemfn) => itemfn.pp(cx),
            | &Item::ForeignMod(ref itemforeignmod) => itemforeignmod.pp(cx),
            | &Item::Impl(ref itemimpl) => itemimpl.pp(cx),
            | &Item::Macro(ref itemmacro) => itemmacro.pp(cx),
            | &Item::Macro2(ref itemmacro2) => itemmacro2.pp(cx),
            | &Item::Mod(ref itemmod) => itemmod.pp(cx),
            | &Item::Static(ref itemstatic) => itemstatic.pp(cx),
            | &Item::Struct(ref itemstruct) => itemstruct.pp(cx),
            | &Item::Trait(ref itemtrait) => itemtrait.pp(cx),
            | &Item::TraitAlias(ref itemtraitalias) => itemtraitalias.pp(cx),
            | &Item::Type(ref itemtype) => itemtype.pp(cx),
            | &Item::Union(ref itemunion) => itemunion.pp(cx),
            | &Item::Use(ref itemuse) => itemuse.pp(cx),
            | &Item::Verbatim(ref tokenstream) => tokenstream.pp(cx),
            | _ => {
                eprintln!("Unsupported variant: {:?}", self);
                self.into_token_stream().to_string()
            },
        }
    }
}

impl PrettyPrint for ItemExternCrate {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        let &Self {
            ref attrs,
            ref vis,
            extern_token: _,
            crate_token: _,
            ref ident,
            ref rename,
            semi_token: _,
        } = self;
        let attrs = if attrs.is_empty() {
            "".into()
        } else {
            attrs
                .iter()
                .map(|attr| format!("{}\n", attr.pp(cx)))
                .join("")
        };
        let vis = vis.pp(cx);
        let indent = cx.indent();
        f!("{attrs}{indent}{vis}extern crate {ident}{rename};",
            rename = if let Some((_, ident)) = rename {
                format!(" as {}", ident.pp(cx))
            } else {
                "".into()
            },
        )
    }
}

impl PrettyPrint for Visibility {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        match self {
            | &Visibility::Inherited => "".into(),
            | &Visibility::Crate(_) => "crate".into(),
            | &Visibility::Public(_) => "pub".into(),
            | &Visibility::Restricted(ref restricted) => {
                let in_ = restricted.in_token.map(|_| "in ").unwrap_or("");
                let path = restricted.path.pp(cx);
                f!("pub({in_}{path})")
            },
        }
    }
}

impl PrettyPrint for ItemUse {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        let &Self {
            ref attrs,
            ref vis,
            use_token: _,
            ref leading_colon,
            ref tree,
            semi_token: _,
        } = self;
        let attrs = if attrs.is_empty() { "".into() } else {
            attrs.iter().map(|attr| format!("{}\n", attr.pp(cx))).join("")
        };
        let mut vis = vis.pp(cx);
        if vis.is_empty().not() { vis.push(' '); }
        let leading_colon = leading_colon.map(|_| "::").unwrap_or("");
        let tree = tree.pp(cx);
        let indent = cx.indent();
        f!("{indent}{attrs}{indent}{vis}use {leading_colon}{tree};")
    }
}

impl PrettyPrint for Ident {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        self.to_string()
    }
}

impl PrettyPrint for UseTree {
    fn pp (self: &'_ Self, cx: &'_ Context)
        -> String
    {
        match self {
            | &UseTree::Path(UsePath {
                ref ident,
                colon2_token: _,
                ref tree,
            }) => {
                let tree = (&**tree).pp(cx);
                format_f!("{ident}::{tree}")
            },
            | &UseTree::Name(ref use_name) => use_name.ident.pp(cx),
            | &UseTree::Rename(ref use_rename) => {
                format!("{ident} as {rename}",
                    ident = use_rename.ident.pp(cx),
                    rename = use_rename.rename.pp(cx),
                )
            },
            | &UseTree::Glob(_) => "*".into(),
            | &UseTree::Group(ref group) => {
                let mb_glob = group.items.iter().find(|x| {
                    if let &UseTree::Glob(_) = x {
                        true
                    } else {
                        false
                    }
                });
                let mb_self = group.items.iter().find(|x| {
                    if let &UseTree::Name(ref name) = x {
                        name.ident == "self"
                    } else {
                        false
                    }
                });
                let same_line = match (mb_glob, mb_self) {
                    | (Some(_), None) => "*,",
                    | (None, Some(_)) => "self,",
                    | (None, None) => "",
                    | (Some(_), Some(_)) => "self, *,",
                };
                let elems = if group.items.is_empty() {
                    "".into()
                } else {
                    let indent = cx.indent();
                    let ref cx = cx.deeper();
                    format!("\n{indent}{}",
                        group
                            .items
                            .iter()
                            .filter(|&x| {
                                let mb_x = Some(x);
                                mb_x != mb_glob && mb_x != mb_self
                            })
                            .map(|x| f!("    {},\n{indent}", x.pp(cx)))
                            .format(""),
                        indent = indent,
                    )
                };
                let indent = cx.indent();
                f!("{{{same_line}{elems}}}")
            },
        }
    }
}
