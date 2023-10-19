use std::{
    cell::OnceCell,
    ptr::{addr_of, addr_of_mut},
    rc::Rc,
};

use ego_tree::NodeRef;
use scraper::{Node, Selector};
use serde::{Deserialize, Serialize};

use crate::{element::unquote, spec, split_docs_link, unwrap_link, unwrap_spanned_link};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AttributeValue {
    pub raw: String,
    pub ty: Option<String>,
    /// Data type docs
    pub docs: Option<String>,
    pub guessed: bool,
    pub verified: bool,
}

fn normalize_attribute_value(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl AttributeValue {
    pub fn scrape(link: &str) -> Self {
        log::trace!("scraping attribute value of: {}", link);

        let spec = spec();

        let selector = Selector::parse(link).expect("invalid attribute link");

        let mut el = spec
            .select(&selector)
            .next()
            .expect("unable to locate attribute definition element")
            .descendants()
            .next()
            .unwrap();

        if !el.has_children() {
            el = el.parent().unwrap();
        }

        fn parse_dl(el: NodeRef<'_, Node>) -> (String, Option<String>) {
            let value = el
                .children()
                .rev()
                .skip_while(|it| !it.value().is_element())
                .next()
                .expect("unable to locate attribute value element");

            let value = value.first_child().expect("empty attribute value tag");

            if let Some(text) = value.value().as_text() {
                (text.to_string(), None)
            } else if let Some(value_el) = value.value().as_element() {
                match value_el.name() {
                    "a" => {
                        let (raw, docs) = unwrap_link(value);
                        (raw, Some(docs))
                    }
                    "em" => (
                        value
                            .children()
                            .next()
                            .expect("empty value content")
                            .value()
                            .as_text()
                            .expect("expected text in em")
                            .to_string(),
                        None,
                    ),
                    _ => todo!("unhandled attribute value element"),
                }
            } else {
                unreachable!("expected attr-value child to be either text or element")
            }
        }

        fn parse_table(el: NodeRef<'_, Node>) -> (String, Option<String>) {
            // TODO: Explicitly select first table
            let mut properties = el.descendants().filter(|it| {
                it.value()
                    .as_element()
                    .map(|it| it.name() == "tr")
                    .unwrap_or_default()
            });

            let value = properties.next().expect("no property table rows");
            // TODO: Check first child

            // FIXME: Seems to produce el contents
            let raw = value
                .last_child()
                .expect("missing table value")
                .descendants()
                .filter_map(|desc| {
                    desc.value()
                        .as_text()
                        .map(|text| {
                            let text = text.to_string();
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                Some(trimmed.to_string())
                            } else {
                                None
                            }
                        })
                        .flatten()
                })
                .collect::<Vec<_>>()
                .join(" ");
            (raw, None)
        }

        fn classed_sibling<'a>(this: NodeRef<'a, Node>, class: &str) -> Option<NodeRef<'a, Node>> {
            this.next_siblings().find(|it| {
                it.value()
                    .as_element()
                    .map(|it| it.classes().any(|it| it == class))
                    .unwrap_or_default()
            })
        }

        match el.value().as_element().unwrap().name() {
            "dt" => {
                let (raw, docs) = parse_dl(el);
                AttributeValue {
                    raw: normalize_attribute_value(&raw),
                    docs,
                    ..Default::default()
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let (raw, docs) = if let Some(prop_def) = classed_sibling(el, "propdef") {
                    parse_table(prop_def)
                } else if let Some(attrib_def) = classed_sibling(el, "adef-list") {
                    let attrib_def = attrib_def
                        .children()
                        .find(|it| {
                            it.value()
                                .as_element()
                                .map(|it| it.name() == "dl")
                                .unwrap_or_default()
                        })
                        .expect("unable to locate attribute definition list");
                    parse_dl(attrib_def)
                } else {
                    panic!("unable to locate attribute information")
                };

                AttributeValue {
                    raw: normalize_attribute_value(&raw),
                    docs,
                    ..Default::default()
                }
            }
            "p" => {
                // this is the worst case where we can't deduce anything from
                // the value
                log::warn!("junk attribute value for: {}", link);

                AttributeValue {
                    raw: "<anything>".to_string(),
                    docs: None,
                    guessed: true,
                    ..Default::default()
                }
            }
            other => {
                todo!(
                    "attribute definition element tag '{}' not implemented",
                    other
                )
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeInterface {
    pub name: String,
    pub value: AttributeValue,
    pub docs: String,
    pub verified: bool,
}

impl AttributeInterface {
    #[inline]
    pub fn from_spanned_link(node: NodeRef<'_, Node>) -> Self {
        Self::new(unwrap_spanned_link(node))
    }

    pub fn new((text, target): (String, String)) -> Self {
        let name = unquote(&text);

        AttributeInterface {
            name,
            value: AttributeValue::scrape(&target),
            docs: split_docs_link(&target),
            verified: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeGroup {
    pub name: String,
    pub bundle_name: String,
    pub attributes: Vec<AttributeInterface>,
    pub docs: String,
    pub verified: bool,
}

static mut GROUPS: Vec<Rc<AttributeGroup>> = Vec::new();
pub fn attribute_groups() -> &'static mut Vec<Rc<AttributeGroup>> {
    unsafe { addr_of_mut!(GROUPS).as_mut().unwrap() }
}

fn bundle_struct_name(name: &str) -> String {
    name.split(' ')
        .map(|it| {
            String::new()
                + (it.as_bytes()[0] as char)
                    .to_uppercase()
                    .to_string()
                    .as_str()
                + &it[1..]
        })
        .collect::<Vec<String>>()
        .join("")
}

impl AttributeGroup {
    pub fn from_link_and_attributes(
        group_link: NodeRef<'_, Node>,
        attributes: Vec<NodeRef<'_, Node>>,
    ) -> Rc<Self> {
        let (text, target) = unwrap_link(group_link);

        if let Some(cached) = attribute_groups().iter().find(|it| it.name == text) {
            return cached.clone();
        }

        log::debug!("Processing {} attributes...", text);
        let bundle_name = bundle_struct_name(&text);
        let attributes = attributes
            .iter()
            .cloned()
            .map(AttributeInterface::from_spanned_link)
            .collect();

        let result = AttributeGroup {
            name: text,
            bundle_name,
            attributes,
            docs: split_docs_link(&target),
            verified: false,
        };

        attribute_groups().push(Rc::new(result));
        unsafe { GROUPS.last().unwrap().clone() }
    }
}

pub(crate) mod serialize_group_named {
    use std::rc::Rc;

    use serde::{
        de::{self},
        Deserializer, Serializer,
    };

    use super::{attribute_groups, AttributeGroup};

    pub fn serialize<S>(value: &Vec<Rc<AttributeGroup>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(
            value
                .iter()
                .map(|it| it.bundle_name.as_str())
                .collect::<Vec<_>>()
                .join(",")
                .as_str(),
        )
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Vec<Rc<AttributeGroup>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bundles: &str = de::Deserialize::deserialize(d)?;
        let bundles_names: Vec<&str> = bundles.split(',').collect();

        let bundles: Vec<Rc<AttributeGroup>> = bundles_names
            .iter()
            .filter_map(|bundle| {
                attribute_groups()
                    .iter()
                    .find(|it| it.bundle_name == *bundle)
                    .cloned()
            })
            .collect();

        if bundles.len() == bundles_names.len() {
            return Ok(bundles);
        } else {
            Err(de::Error::custom("couldn't find required bundles in cache"))
        }
    }
}
