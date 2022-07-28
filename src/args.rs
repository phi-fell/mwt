use quote::{quote, ToTokens};
use syn::fold::{fold_expr, fold_path_arguments, fold_type, fold_type_reference, Fold};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;

use syn::{
    Expr, GenericArgument, Ident, ItemFn, PathArguments, PathSegment, Token, Type, TypeReference,
};

pub struct Args {
    mut_version: bool,
    ignore_self: bool,
    ident_string: String,
    type_string: String,
    type_switch_string: String,
    ref_string: String,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut ignore_self = false;
        let ident_string = "mwt".to_owned();
        let type_string = "Mwt".to_owned();
        let type_switch_string = "MwtAlt".to_owned();
        let ref_string = "Mwt".to_owned();
        for arg in Punctuated::<Ident, Token![,]>::parse_terminated(input)? {
            match arg.to_string().as_str() {
                "ignore_self" => ignore_self = true,
                _ => return Err(input.error("invalid argument!")),
            }
        }
        Ok(Args {
            mut_version: false,
            ignore_self,
            ident_string,
            type_string,
            type_switch_string,
            ref_string,
        })
    }
}

impl Args {
    pub fn set_ident_str(&mut self, s: String) {
        self.ident_string = s;
    }
    pub fn set_type_str(&mut self, s: String) {
        self.type_string = s;
    }
    pub fn set_type_switch_str(&mut self, s: String) {
        self.type_string = s;
    }
    pub fn set_ref_str(&mut self, s: String) {
        self.ref_string = s;
    }

    pub fn convert_fn(&mut self, mut_version: bool, mut root: ItemFn) -> ItemFn {
        self.mut_version = mut_version;

        if !self.mut_version && !self.ignore_self {
            root.sig.inputs.iter_mut().for_each(|arg| match arg {
                syn::FnArg::Receiver(self_arg) => self_arg.mutability = None,
                syn::FnArg::Typed(_) => {}
            });
        }

        self.fold_item_fn(root)
    }
}

impl Fold for Args {
    fn fold_ident(&mut self, i: Ident) -> Ident {
        if self.mut_version {
            Ident::new(&i.to_string().replace(&self.ident_string, "mut"), i.span())
        } else {
            Ident::new(
                i.to_string()
                    .replace(&format!("_{}_", self.ident_string), "_")
                    .trim_start_matches(&format!("{}_", self.ident_string))
                    .trim_end_matches(&format!("_{}", self.ident_string)),
                i.span(),
            )
        }
    }
    fn fold_type(&mut self, t: Type) -> Type {
        match t {
            Type::Path(t) => {
                if t.path.leading_colon.is_none() && t.path.segments.len() == 1 {
                    // borrow path segments since we don't want to destructively modify t yet
                    let ps = t.path.segments.first().unwrap(/* len() is checked above */);
                    if ps.ident == self.type_switch_string {
                        // we can now move out of t, since we will either be returning something else, or panicking
                        let ps =
                            t.path.segments.into_iter().next().unwrap(/* len() is checked above */);
                        if let PathArguments::AngleBracketed(args) = ps.arguments {
                            if args.args.len() == 2 {
                                if self.mut_version {
                                    match args.args.into_iter().next().unwrap(/* len() is checked above */)
                                    {
                                        GenericArgument::Type(t) => return t,
                                        _ => panic!(
                                            "First argument to `{0}` is invalid! Must be a type (not a lifetime/constraint/etc)!",
                                            self.type_switch_string
                                        ),
                                    }
                                } else {
                                    match args.args.into_iter().next_back().unwrap(/* len() is checked above */)
                                    {
                                        GenericArgument::Type(t) => return t,
                                        _ => panic!(
                                            "Second argument to `{0}` is invalid! Must be a type (not a lifetime/constraint/etc)!",
                                            self.type_switch_string
                                        ),
                                    }
                                }
                            } else {
                                panic!(
                                    "`{0}` needs exactly 2 type parameters, e.g. `{0}<A, B>`",
                                    self.type_switch_string
                                );
                            }
                        } else {
                            panic!(
                                "`{0}` needs bracketed type parameters, e.g. `{0}<A, B>`",
                                self.type_switch_string
                            );
                        }
                    }
                }
                Type::Path(self.fold_type_path(t))
            }
            _ => fold_type(self, t),
        }
    }
    fn fold_path_segment(&mut self, ps: PathSegment) -> PathSegment {
        let s = if self.mut_version {
            ps.ident
                .to_string()
                .replace(&self.type_string, "Mut")
                .replace(&self.ident_string, "mut")
        } else {
            ps.ident
                .to_string()
                .replace(&self.type_string, "")
                .replace(&format!("_{}_", self.ident_string), "_")
                .trim_start_matches(&format!("{}_", self.ident_string))
                .trim_end_matches(&format!("_{}", self.ident_string))
                .to_owned()
        };
        let ident = Ident::new(&s, ps.ident.span());
        PathSegment {
            ident,
            arguments: fold_path_arguments(self, ps.arguments),
        }
    }
    fn fold_type_reference(&mut self, tr: TypeReference) -> TypeReference {
        match *tr.elem.clone() {
            Type::Path(tp) => {
                if tp.path.segments.len() == 1 {
                    if let Some(seg) = tp.path.segments.first() {
                        if seg.ident == self.ref_string {
                            if let PathArguments::AngleBracketed(args) = &seg.arguments {
                                if args.args.len() == 1 {
                                    if let Some(GenericArgument::Type(t)) = args.args.first() {
                                        return TypeReference {
                                            and_token: tr.and_token,
                                            lifetime: tr.lifetime,
                                            mutability: if self.mut_version {
                                                Some(Token![mut](seg.ident.span()))
                                            } else {
                                                None
                                            },
                                            elem: Box::new(t.clone()),
                                        };
                                    }
                                }
                            }
                            panic!("{0} needs one type param e.g. &{0}<T>", self.ref_string)
                        }
                    }
                }
                fold_type_reference(self, tr)
            }
            _ => fold_type_reference(self, tr),
        }
    }
    fn fold_expr(&mut self, e: Expr) -> Expr {
        match e {
            Expr::Block(eb) => {
                if eb.attrs.iter().any(|a| a.path.is_ident("if_mut")) {
                    if self.mut_version {
                        let statements = eb.block.stmts;
                        Expr::Verbatim(quote! {
                            #(#statements);*
                        })
                    } else {
                        Expr::Verbatim(quote! {})
                    }
                } else if eb.attrs.iter().any(|a| a.path.is_ident("not_mut")) {
                    if self.mut_version {
                        Expr::Verbatim(quote! {})
                    } else {
                        let statements = eb.block.stmts;
                        Expr::Verbatim(quote! {
                            #(#statements);*
                        })
                    }
                } else {
                    Expr::Block(self.fold_expr_block(eb))
                }
            }
            Expr::Call(e) => {
                if e.func.to_token_stream().to_string() == self.ident_string {
                    let inner = e.args;
                    if self.mut_version {
                        Expr::Verbatim(quote! {
                            mut #inner
                        })
                    } else {
                        Expr::Verbatim(quote! {
                            #inner
                        })
                    }
                } else {
                    Expr::Call(self.fold_expr_call(e))
                }
            }
            _ => fold_expr(self, e),
        }
    }
}
