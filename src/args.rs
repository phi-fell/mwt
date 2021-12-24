use quote::quote;
use syn::fold::Fold;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;

use syn::{Expr, GenericArgument, Ident, ItemFn, PathArguments, Token, Type, TypeReference};

pub struct Args {
    mut_version: bool,
    ignore_self: bool,
    ident_string: String,
    ref_string: String,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut ignore_self = false;
        let ident_string = "mwt".to_owned();
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
            ref_string,
        })
    }
}

impl Args {
    pub fn set_ident_str(&mut self, s: String) {
        self.ident_string = s;
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
                tr
            }
            _ => tr,
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
            Expr::Macro(e) => {
                if e.mac.path.is_ident(&self.ident_string) {
                    let inner = e.mac.tokens;
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
                    Expr::Macro(self.fold_expr_macro(e))
                }
            }
            Expr::Array(a) => Expr::Array(self.fold_expr_array(a)),
            Expr::Assign(e) => Expr::Assign(self.fold_expr_assign(e)),
            Expr::AssignOp(e) => Expr::AssignOp(self.fold_expr_assign_op(e)),
            Expr::Async(e) => Expr::Async(self.fold_expr_async(e)),
            Expr::Await(e) => Expr::Await(self.fold_expr_await(e)),
            Expr::Binary(e) => Expr::Binary(self.fold_expr_binary(e)),
            Expr::Box(e) => Expr::Box(self.fold_expr_box(e)),
            Expr::Break(e) => Expr::Break(self.fold_expr_break(e)),
            Expr::Call(e) => Expr::Call(self.fold_expr_call(e)),
            Expr::Cast(e) => Expr::Cast(self.fold_expr_cast(e)),
            Expr::Closure(e) => Expr::Closure(self.fold_expr_closure(e)),
            Expr::Continue(e) => Expr::Continue(self.fold_expr_continue(e)),
            Expr::Field(e) => Expr::Field(self.fold_expr_field(e)),
            Expr::ForLoop(e) => Expr::ForLoop(self.fold_expr_for_loop(e)),
            Expr::Group(e) => Expr::Group(self.fold_expr_group(e)),
            Expr::If(e) => Expr::If(self.fold_expr_if(e)),
            Expr::Index(e) => Expr::Index(self.fold_expr_index(e)),
            Expr::Let(e) => Expr::Let(self.fold_expr_let(e)),
            Expr::Lit(e) => Expr::Lit(self.fold_expr_lit(e)),
            Expr::Loop(e) => Expr::Loop(self.fold_expr_loop(e)),
            Expr::Match(e) => Expr::Match(self.fold_expr_match(e)),
            Expr::MethodCall(e) => Expr::MethodCall(self.fold_expr_method_call(e)),
            Expr::Paren(e) => Expr::Paren(self.fold_expr_paren(e)),
            Expr::Path(e) => Expr::Path(self.fold_expr_path(e)),
            Expr::Range(e) => Expr::Range(self.fold_expr_range(e)),
            Expr::Reference(e) => Expr::Reference(self.fold_expr_reference(e)),
            Expr::Repeat(e) => Expr::Repeat(self.fold_expr_repeat(e)),
            Expr::Return(e) => Expr::Return(self.fold_expr_return(e)),
            Expr::Struct(e) => Expr::Struct(self.fold_expr_struct(e)),
            Expr::Try(e) => Expr::Try(self.fold_expr_try(e)),
            Expr::TryBlock(e) => Expr::TryBlock(self.fold_expr_try_block(e)),
            Expr::Tuple(e) => Expr::Tuple(self.fold_expr_tuple(e)),
            Expr::Type(e) => Expr::Type(self.fold_expr_type(e)),
            Expr::Unary(e) => Expr::Unary(self.fold_expr_unary(e)),
            Expr::Unsafe(e) => Expr::Unsafe(self.fold_expr_unsafe(e)),
            Expr::While(e) => Expr::While(self.fold_expr_while(e)),
            Expr::Yield(e) => Expr::Yield(self.fold_expr_yield(e)),
            Expr::Verbatim(e) => Expr::Verbatim(e),
            _ => e,
        }
    }
}
