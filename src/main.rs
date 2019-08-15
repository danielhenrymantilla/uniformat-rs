#![feature(core_intrinsics, specialization)]
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
use syn::{*,
    parse::Parse,
};

trait PrettyPrint {
    fn pp (self: &'_ Self)
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
        println_f!("{source_file:?}\n\n{}", source_file.pp());
    })}

    if let Err(msg) = main() {
        eprintln_f!("{msg}");
        ::std::process::exit(1);
    }
}

impl<T : fmt::Debug> PrettyPrint for T {
    default
    fn pp (self: &'_ Self)
        -> String
    {
        let typename: &str = unsafe {
            ::std::intrinsics::type_name::<T>()
        };
        f!("<TODO: {typename}.pp()>", )
    }
}

impl PrettyPrint for File {
    fn pp (self: &'_ Self)
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
                            .map(PrettyPrint::pp)
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
                            .map(PrettyPrint::pp)
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
    fn pp (self: &'_ Self) -> String
    {
        let &Self {
            pound_token: _,
            ref style,
            bracket_token: _,
            ref path,
            ref tokens,
        } = self;
        format!(
            "#{pound}[{inner}]",

            pound = if let &AttrStyle::Inner(_) = style {
                "!"
            } else {
                ""
            },

            inner = if let Ok(meta) = self.parse_meta() {
                meta.pp()
            } else {
                let path = path.pp();
                f!("{path} {tokens}")
            },
        )
    }
}

impl PrettyPrint for Path {
    fn pp (self: &'_ Self) -> String
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
                            arguments = segment.arguments.pp(),
                        )
                    })
                    .format("::")
            ,
        )
    }
}

impl PrettyPrint for PathArguments {
    fn pp (self: &'_ Self) -> String
    {
        use PathArguments::*;
        match self {
            | &None => "".into(),
            | &AngleBracketed(ref angle_bracketed) => {
                unimplemented!("TODO: AngleBracketed")
            },
            | &Parenthesized(ParenthesizedGenericArguments{
                paren_token: _,
                ref inputs,
                ref output,
            }) => {
                format!("({args}){ret_ty}",
                    ret_ty = match output {
                        | &ReturnType::Default => "".into(),
                        | &ReturnType::Type(_, ref ty) => ty.pp(),
                    },
                    args =
                        inputs
                            .iter()
                            .map(PrettyPrint::pp)
                            .format(", ")
                    ,
                )
            },
        }
    }
}

impl PrettyPrint for Meta {
    fn pp (self: &'_ Self) -> String
    {
        match self {
            | &Meta::Path(ref path) => path.pp(),
            | &Meta::NameValue(ref name_value) => {
                let path = name_value.path.pp();
                let lit = name_value.lit.pp();
                f!("{path} = {lit}")
            },
            | &Meta::List(ref list) => {
                let path = list.path.pp();
                f!("{path}({nested})", nested = {
                    list.nested
                        .iter()
                        .map(|nested_meta| match nested_meta {
                            | &NestedMeta::Meta(ref meta) => meta.pp(),
                            | &NestedMeta::Lit(ref lit) => lit.pp(),
                        })
                        .format(", ")
                })
            },
        }
    }
}

impl PrettyPrint for Lit {
    fn pp (self: &'_ Self) -> String
    {
        self.clone().into_token_stream().to_string()
    }
}

impl PrettyPrint for Item {
    fn pp (self: &'_ Self) -> String
    {
        match self {
            | &Item::Const(ref itemconst) => itemconst.pp(),
            | &Item::Enum(ref itemenum) => itemenum.pp(),
            | &Item::ExternCrate(ref itemexterncrate) => itemexterncrate.pp(),
            | &Item::Fn(ref itemfn) => itemfn.pp(),
            | &Item::ForeignMod(ref itemforeignmod) => itemforeignmod.pp(),
            | &Item::Impl(ref itemimpl) => itemimpl.pp(),
            | &Item::Macro(ref itemmacro) => itemmacro.pp(),
            | &Item::Macro2(ref itemmacro2) => itemmacro2.pp(),
            | &Item::Mod(ref itemmod) => itemmod.pp(),
            | &Item::Static(ref itemstatic) => itemstatic.pp(),
            | &Item::Struct(ref itemstruct) => itemstruct.pp(),
            | &Item::Trait(ref itemtrait) => itemtrait.pp(),
            | &Item::TraitAlias(ref itemtraitalias) => itemtraitalias.pp(),
            | &Item::Type(ref itemtype) => itemtype.pp(),
            | &Item::Union(ref itemunion) => itemunion.pp(),
            | &Item::Use(ref itemuse) => itemuse.pp(),
            | &Item::Verbatim(ref tokenstream) => tokenstream.pp(),
            | _ => format!("{:?}", self),
        }
    }
}

impl PrettyPrint for ItemExternCrate {
    fn pp (self: &'_ Self) -> String
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
        let attrs = if attrs.is_empty() { "".into() } else {
            attrs.iter().map(|attr| format!("{}\n", attr.pp())).join("")
        };
        let vis = vis.pp();
        f!("{attrs}{vis}extern crate {ident}{rename};",
            rename = if let Some((_, ident)) = rename {
                format!(" as {}", ident.pp())
            } else {
                "".into()
            },
        )
    }
}

impl PrettyPrint for Visibility {
    fn pp (self: &'_ Self) -> String
    {
        match self {
            | &Visibility::Inherited => "".into(),
            | &Visibility::Crate(_) => "crate".into(),
            | &Visibility::Public(_) => "pub".into(),
            | &Visibility::Restricted(ref restricted) => {
                let in_ = restricted.in_token.map(|_| "in ").unwrap_or("");
                let path = restricted.path.pp();
                f!("pub({in_}{path})")
            },
        }
    }
}

impl PrettyPrint for ItemUse {
    fn pp (self: &'_ Self) -> String
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
            attrs.iter().map(|attr| format!("{}\n", attr.pp())).join("")
        };
        let vis = vis.pp();
        let leading_colon = leading_colon.map(|_| "::").unwrap_or("");
        let tree = tree.pp();
        f!("{attrs}{vis}use {leading_colon}{tree};")
    }
}

impl PrettyPrint for Ident {
    fn pp (self: &'_ Self) -> String
    {
        self.to_string()
    }
}

impl PrettyPrint for UseTree {
    fn pp (self: &'_ Self) -> String
    {
        match self {
            | &UseTree::Path(UsePath {
                ref ident,
                colon2_token: _,
                ref tree,
            }) => {
                let tree = (&**tree).pp();
                format_f!("{ident}::{tree}")
            },
            | &UseTree::Name(ref use_name) => use_name.ident.pp(),
            | &UseTree::Rename(ref use_rename) => {
                format!("{ident} as {rename}",
                    ident = use_rename.ident.pp(),
                    rename = use_rename.rename.pp(),
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
                    format!("\n{}",
                        group
                            .items
                            .iter()
                            .filter(|&x| {
                                Some(x) != mb_glob && Some(x) != mb_self
                            })
                            .map(|x| format!("    {},\n", x.pp()))
                            .format("")
                    )
                };
                f!("{{{same_line}{elems}}}")
            },
        }
    }
}

impl PrettyPrint for UsePath {
    // fn pp (self: &'_ Self) -> String
    // {
    //     match self {
    //         | &UseTree::Path(ref path) => path.pp(),
    //         | &UseTree::Name(ref use_name) => use_name.ident.pp(),
    //         | &UseTree::Rename(ref use_rename) => {
    //             format!("{ident} as {rename}",
    //                 ident = use_rename.ident.pp(),
    //                 rename = use_rename.rename.pp(),
    //             )
    //         },
    //         | &UseTree::Glob(_) => "*".into(),
    //         | &UseTree::Group(ref group) => {
    //             format!("{{\n    {elems}\n}}", elems = {
    //                 group
    //                     .items
    //                     .iter()
    //                     .map(PrettyPrint::pp)
    //                     .format(",\n    ")
    //             })
    //         },
    //     }
    // }
}