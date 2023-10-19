use syn::parse::Parse;
use syn::*;

#[derive(Default)]
pub struct KVPairs {
    pub data: Vec<(Ident, Expr)>,
}

impl KVPairs {
    /// `None` if attribute with `name` is not present, `Err` if KVPairs is
    /// invalid, otherwise `Some(Ok(kv_pairs))`.
    pub fn from_field_attribute(field: &Field, name: impl AsRef<str>) -> Option<Result<Self>> {
        field.attrs.iter().find_map(|attr| match &attr.meta {
            Meta::Path(path) => {
                if path.is_ident(name.as_ref()) {
                    Some(Ok(KVPairs::default()))
                } else {
                    None
                }
            }
            Meta::List(list) => {
                if list.path.is_ident(name.as_ref()) {
                    if !matches!(list.delimiter, MacroDelimiter::Brace(_)) {
                        Some(Err(Error::new_spanned(
                            &list.path,
                            "attribute expects brace delimiters",
                        )))
                    } else {
                        Some(parse2(list.tokens.clone()))
                    }
                } else {
                    None
                }
            }
            Meta::NameValue(_) => None,
        })
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&Expr> {
        self.data.iter().find_map(|(ident, it)| {
            if *ident == name.as_ref() {
                Some(it)
            } else {
                None
            }
        })
    }
}

impl Parse for KVPairs {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let mut data = Vec::new();
        while !input.is_empty() {
            let key = input.parse::<Ident>()?;
            if let Some((prev, _)) = data.iter().find(|(it, _)| *it == key) {
                let mut err = Error::new_spanned(key, "duplicate key");
                err.combine(Error::new_spanned(prev, "previously defined here"));
                return Err(err);
            }
            input.parse::<token::Colon>()?;
            let value = input.parse::<Expr>()?;
            data.push((key, value));
            if !input.is_empty() {
                input.parse::<token::Comma>()?;
            }
        }
        Ok(KVPairs { data })
    }
}

pub enum ValueExpression {
    Pass,
    Transform(Expr),
    Literal(LitByteStr),
}

impl ValueExpression {
    pub fn handle(&self, name: &Ident, attrib_name: &LitByteStr) -> Block {
        let key = {
            let mut name = attrib_name.value();
            name.extend_from_slice(b"=\"");
            LitByteStr::new(name.as_slice(), attrib_name.span())
        };

        match self {
            ValueExpression::Pass => {
                parse_quote! {{
                    if wrote_any_attributes {
                        writer.write(b" ")?;
                    }
                    writer.write( #key )?;
                    crate::io::Writable::write_to( #name , writer, settings)?;
                    writer.write(b"\"")?;
                    wrote_any_attributes = true;
                }}
            }
            ValueExpression::Transform(expr) => {
                parse_quote! {{
                    if wrote_any_attributes {
                        writer.write(b" ")?;
                    }
                    writer.write( #key )?;
                    writer.write( #expr )?;
                    writer.write(b"\"")?;
                    wrote_any_attributes = true;
                }}
            }
            ValueExpression::Literal(literal) => {
                let literal = {
                    let mut value = key.value();
                    value.extend_from_slice(literal.value().as_ref());
                    value.extend_from_slice(b"\"");
                    LitByteStr::new(value.as_slice(), literal.span())
                };

                parse_quote! {{
                    if wrote_any_attributes {
                        writer.write(b" ")?;
                    }
                    writer.write( #literal )?;
                    wrote_any_attributes = true;
                }}
            }
        }
    }
}

pub enum Check {
    None,
    Optional,
    Default,
    Other(ExprClosure),
}

impl Check {
    pub fn wrapped(&self, name: &Ident, inner: Block) -> Expr {
        match self {
            Check::None => {
                let mut block = inner;

                // converge locals
                block.stmts.insert(
                    0,
                    parse_quote! {
                        let #name = &self. #name;
                    },
                );

                Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block,
                })
            }
            Check::Optional => {
                parse_quote! {
                    if let Some(#name) = &self. #name {
                        #inner
                    }
                }
            }
            Check::Default => {
                parse_quote! {
                    if self. #name == Default::default() {
                        #inner
                    }
                }
            }
            Check::Other(check) => {
                parse_quote! {
                    if (#check)(&self. #name) {
                        #inner
                    }
                }
            }
        }
    }
}

pub struct XmlAttribute {
    pub name: Ident,
    pub attrib_name: LitByteStr,
    pub ty: Type,
    pub check: Check,
    pub value_expr: ValueExpression,
}

fn is_option(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => {
            let last = match path.path.segments.last() {
                Some(it) => it,
                None => return false,
            };

            let one_arg = match &last.arguments {
                PathArguments::AngleBracketed(args) => args.args.len() == 1,
                _ => false,
            };

            last.ident == "Option" && one_arg
        }
        _ => false,
    }
}

impl XmlAttribute {
    pub fn new(field: &Field) -> Option<Result<Self>> {
        let pairs = match KVPairs::from_field_attribute(field, "xml_attribute")? {
            Ok(it) => it,
            Err(err) => return Some(Err(err)),
        };

        let name = match &field.ident {
            Some(it) => it.clone(),
            None => return Some(Err(Error::new_spanned(field, "expected an identifier"))),
        };
        let ty = field.ty.clone();

        let value_expr = {
            if let Some(transform) = pairs.get("transform") {
                ValueExpression::Transform(transform.clone())
            } else if let Some(literal) = pairs.get("literal") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::ByteStr(literal),
                    ..
                }) = literal
                {
                    ValueExpression::Literal(literal.clone())
                } else {
                    return Some(Err(Error::new_spanned(
                        literal,
                        "expected a byte string literal",
                    )));
                }
            } else {
                ValueExpression::Pass
            }
        };

        let check = if let Some(check_expr) = pairs.get("check") {
            match check_expr {
                Expr::Closure(closure) => Check::Other(closure.clone()),
                Expr::Path(path) => {
                    if path.path.is_ident("Default") {
                        Check::Default
                    } else if path.path.is_ident("Option") {
                        if !is_option(&ty) {
                            return Some(Err(Error::new_spanned(
                                check_expr,
                                "'Option' only works on Option type",
                            )));
                        }
                        Check::Optional
                    } else if path.path.is_ident("None") {
                        Check::None
                    } else {
                        return Some(Err(Error::new_spanned(
                            check_expr,
                            "expected one of: 'Default', 'Option', 'None'",
                        )));
                    }
                }
                _ => {
                    return Some(Err(Error::new_spanned(
                        check_expr,
                        "expected a closure or one of: 'Default', 'Option', 'None'",
                    )))
                }
            }
        } else {
            if is_option(&ty) {
                Check::Optional
            } else {
                Check::None
            }
        };

        let attrib_name = if let Some(name) = pairs.get("name") {
            match name {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(name),
                    ..
                }) => LitByteStr::new(name.value().as_bytes(), name.span()),
                _ => {
                    return Some(Err(Error::new_spanned(name, "expected a string literal")));
                }
            }
        } else {
            LitByteStr::new(name.to_string().as_bytes(), name.span())
        };

        Some(Ok(XmlAttribute {
            name,
            attrib_name,
            ty,
            check,
            value_expr,
        }))
    }

    pub fn generate_write_expr(self) -> Expr {
        let inner = self.value_expr.handle(&self.name, &self.attrib_name);
        self.check.wrapped(&self.name, inner)
    }
}

pub struct XmlAttributeBundle {
    pub name: Ident,
}

impl XmlAttributeBundle {
    pub fn new(field: &Field) -> Option<Result<XmlAttributeBundle>> {
        let _pairs = match KVPairs::from_field_attribute(field, "xml_attribute_bundle")? {
            Ok(it) => it,
            Err(err) => return Some(Err(err)),
        };

        let name = match &field.ident {
            Some(it) => it.clone(),
            None => return Some(Err(Error::new_spanned(field, "expected an identifier"))),
        };

        Some(Ok(XmlAttributeBundle { name }))
    }

    pub fn generate_write_expr(&self) -> Expr {
        let name = &self.name;
        parse_quote! {{
            if wrote_any_attributes {
                writer.write(b" ")?;
            }
            wrote_any_attributes |= self. #name . write_attributes(writer, settings)?;
        }}
    }
}
