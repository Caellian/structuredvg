use std::{cell::OnceCell, collections::HashMap, rc::Rc};

use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    attribute::{AttributeGroup, AttributeInterface},
    spec, spec_chapter, split_docs_link,
};

pub struct ElInfo {
    pub tag_name: String,
    pub rust_name: String,
    pub module: String,
    pub section: String,
}

#[inline]
fn el_rust_name(name: &str) -> String {
    let capitalized = name
        .split('-')
        .map(|it| {
            String::new()
                + (it.as_bytes()[0] as char)
                    .to_uppercase()
                    .to_string()
                    .as_str()
                + &it[1..]
        })
        .collect::<Vec<String>>()
        .join("");
    "Element".to_string() + capitalized.as_str()
}

pub fn unquote(s: &str) -> String {
    let name_len = s.chars().count();
    s.chars().skip(1).take(name_len - 2).collect()
}

pub fn get_element_info() -> Vec<ElInfo> {
    let index = spec_chapter("eltindex").expect("unable to find Element Index chapter");
    let mut result = Vec::new();
    let elements = Selector::parse("ul li a").expect("invalid element selector");

    for el in index.select(&elements) {
        let section = el
            .value()
            .attr("href")
            .expect("missing element link")
            .to_string();

        let module = section
            .split_once('-')
            .expect("can't determine element module")
            .0
            .to_string();

        let name = el
            .descendants()
            .last()
            .expect("link empty")
            .value()
            .as_text()
            .expect("final link descendant not text")
            .to_string();
        let name_len = name.chars().count();
        let name: String = name.chars().skip(1).take(name_len - 2).collect();
        if !name.is_ascii() {
            log::warn!("non-ascii element name: {}", name);
        }

        result.push(ElInfo {
            rust_name: el_rust_name(&name),
            tag_name: name,
            module,
            section,
        })
    }

    result
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SummaryInfo {
    has_content: bool,
    #[serde(with = "crate::attribute::serialize_group_named")]
    attribute_groups: Vec<Rc<AttributeGroup>>,
    context_attributes: Vec<AttributeInterface>,
    dom_interfaces: Vec<String>,
}

impl SummaryInfo {
    pub fn parse(element: ElementRef<'_>) -> Self {
        let sections = Selector::parse("dl dd").unwrap();
        let mut sections = element.select(&sections).skip(1);

        let has_content = sections
            .next()
            .expect("missing content model summary section")
            .first_child()
            .expect("empty content model summary section")
            .value()
            .as_text()
            .map(|it| !it.to_string().to_lowercase().contains("empty"))
            .unwrap_or_default();

        let mut attribute_groups = vec![];
        let mut context_attributes = vec![];

        let attributes = sections
            .next()
            .expect("missing attributes summary section")
            .first_child()
            .expect("empty attributes summary section")
            .children();

        for li in attributes {
            let mut children = li.children();
            log::trace!(
                "processing li: {:?}",
                children.clone().map(|it| it.value()).collect::<Vec<_>>()
            );

            let link_el = children.next().unwrap();

            match li.children().count() {
                1 => {
                    let attr = AttributeInterface::from_spanned_link(link_el.clone());
                    context_attributes.push(attr);
                }
                2 => {
                    let expanding = children.next().unwrap();

                    log::trace!("expanding span: {:?}", expanding.value());
                    let children = expanding
                        .children()
                        .filter(|it| it.value().is_element())
                        .collect();

                    let group = AttributeGroup::from_link_and_attributes(link_el, children);
                    attribute_groups.push(group);
                }
                _ => {
                    unreachable!("attribute li contains more than 2 children")
                }
            }
        }

        // TODO: DOM interface parsing
        let dom_interfaces = vec![];

        SummaryInfo {
            has_content,
            attribute_groups,
            context_attributes,
            dom_interfaces,
        }
    }
}

fn element_summary(tag_name: impl AsRef<str>) -> Option<ElementRef<'static>> {
    static mut SECTIONS: OnceCell<HashMap<String, ElementRef<'static>>> = OnceCell::new();

    unsafe {
        SECTIONS.get_or_init(|| {
            log::debug!("Processing element summaries...");
            let mut sections = HashMap::new();

            let spec = spec();
            let selector = Selector::parse(".element-summary").unwrap();
            let summaries = spec.select(&selector);

            for summary in summaries {
                if let Some(text) = summary
                    .select(&Selector::parse("span.element-name").unwrap())
                    .next()
                    .and_then(|it| it.first_child())
                {
                    if let Some(text) = text.value().as_text() {
                        let section_tag = unquote(text.to_string().as_str());

                        sections.insert(section_tag, summary);
                    }
                }
            }

            sections
        });
    }

    unsafe {
        SECTIONS
            .get()
            .and_then(|it| it.get(tag_name.as_ref()))
            .cloned()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementInterface {
    pub name: String,
    pub tag_name: String,
    pub module: String,
    #[serde(flatten)]
    pub summary_info: SummaryInfo,
    pub docs: String,
    pub verified: bool,
}

impl ElementInterface {
    pub fn build(info: ElInfo) -> Self {
        let section = match element_summary(&info.tag_name) {
            Some(it) => it,
            None => panic!("unable to locate element summary for: '{}'", info.tag_name),
        };

        ElementInterface {
            name: info.rust_name,
            tag_name: info.tag_name,
            module: info.module,
            summary_info: SummaryInfo::parse(section),
            docs: split_docs_link(&info.section),
            verified: false,
        }
    }
}
