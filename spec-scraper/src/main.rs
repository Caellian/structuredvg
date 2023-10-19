use std::{
    cell::OnceCell, collections::HashMap, hash::Hash, mem::MaybeUninit, path::PathBuf, sync::Once,
};

use attribute::attribute_groups;
use ego_tree::NodeRef;
use element::{get_element_info, unquote};
use scraper::{Element, ElementRef, Html, Node, Selector};
use serde::Serialize;

use crate::element::ElementInterface;

mod attribute;
mod element;
mod util;

const BASE_SPEC_PATH: &str = "https://www.w3.org/TR/SVG11/";
const SPEC_PATH: &str = "https://www.w3.org/TR/SVG11/single-page.html";

const SPEC_CACHE_PATH: &str = "./target/codegen/spec.html";
const DATA_DIR: &str = "./data/";
const ATTR_GROUP_PATH: &str = "./data/attribute_groups.json";
const ELEM_PATH: &str = "./data/elements.json";

fn spec() -> &'static Html {
    static mut PAGE_CACHE: OnceCell<Html> = OnceCell::new();

    unsafe {
        PAGE_CACHE.get_or_init(|| {
            let local = match std::fs::read_to_string(SPEC_CACHE_PATH) {
                Ok(it) => {
                    log::info!("Loaded cached specification.");
                    it
                }
                Err(_) => {
                    log::info!("Downloading: {}", SPEC_PATH);
                    let resp = reqwest::blocking::get(SPEC_PATH)
                        .expect("unable to get page response")
                        .text()
                        .expect("invalid page response");
                    std::fs::create_dir_all(PathBuf::from(SPEC_CACHE_PATH).parent().unwrap())
                        .unwrap();
                    std::fs::write(SPEC_CACHE_PATH, resp.as_str()).unwrap();
                    log::info!("Downloaded and cached specification.");
                    resp.to_string()
                }
            };

            Html::parse_document(&local)
        })
    }
}

fn spec_chapter(id: impl AsRef<str>) -> Option<ElementRef<'static>> {
    let spec = spec();
    spec.select(&Selector::parse(format!("div#chapter-{}", id.as_ref()).as_str()).unwrap())
        .next()
}

fn heading_section(id: impl AsRef<str>) -> Option<Vec<NodeRef<'static, Node>>> {
    let spec = spec();
    let title = spec
        .select(&Selector::parse(format!("#{}", id.as_ref()).as_str()).unwrap())
        .next()?;
    let tag = title.value().name();

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some(
            title
                .parent_element()
                .unwrap()
                .children()
                .skip_while(|it| it.value().as_element() != Some(title.value()))
                .take_while(|it| {
                    !it.value().is_element() || it.value().as_element().unwrap().name() != tag
                })
                .collect(),
        ),
        _ => None,
    }
}

#[inline]
fn unwrap_spanned_link(node: NodeRef<'_, Node>) -> (String, String) {
    log::trace!("unwrapping spanned link: {:?}", node.value());

    let target = node
        .value()
        .as_element()
        .expect("node not an element")
        .attr("href")
        .expect("can't find link href attribute")
        .to_string();

    let text = node
        .first_child()
        .expect("node doesn't contain a span")
        .first_child()
        .expect("span is empty")
        .value()
        .as_text()
        .expect("span content isn't text")
        .to_string();

    (text, target)
}

fn unwrap_link(node: NodeRef<'_, Node>) -> (String, String) {
    let target = node
        .value()
        .as_element()
        .expect("node not an element")
        .attr("href")
        .expect("can't find link href attribute")
        .to_string();

    let text = node
        .first_child()
        .expect("link is empty")
        .value()
        .as_text()
        .expect("link content isn't text")
        .to_string();

    (text, target)
}

fn split_docs_link(section: &str) -> String {
    let mut parts = section[1..].split('-');
    BASE_SPEC_PATH.to_string() + parts.next().unwrap() + ".html#" + parts.next().unwrap()
}

fn scrape() {
    std::fs::create_dir_all(DATA_DIR).expect("can't create data directory");

    let elements = get_element_info();

    let elements: HashMap<_, _> = elements
        .into_iter()
        .map(|info| {
            log::info!(
                "Processing element '{}' ({})",
                &info.rust_name,
                &info.section
            );
            let interface = ElementInterface::build(info);
            (interface.tag_name.clone(), interface)
        })
        .collect();

    let elem = serde_json::to_string_pretty(&elements).expect("unable to serialize elements");
    std::fs::write(ELEM_PATH, elem).expect("unable to store elements");

    let attr_groups = serde_json::to_string_pretty(
        &attribute_groups()
            .iter()
            .map(|it| {
                let group = (*it).as_ref();
                (group.bundle_name.clone(), group)
            })
            .collect::<HashMap<_, _>>(),
    )
    .expect("unable to serialize attribute groups");
    std::fs::write(ATTR_GROUP_PATH, attr_groups).expect("unable to store attribute groups");
}

fn generate() {
    
}

fn main() {
    env_logger::init();

    scrape()
}
