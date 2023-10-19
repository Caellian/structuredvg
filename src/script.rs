use std::borrow::Cow;

use structuredvg_macros::BundleAttributes;

/// Event attributes that can be specified on most
/// [graphics elements](https://www.w3.org/TR/SVG11/intro.html#TermGraphicsElement)
/// and
/// [container elements](https://www.w3.org/TR/SVG11/intro.html#TermContainerElement).
///
/// Values of all of these are [`<anything>`](https://www.w3.org/TR/SVG11/types.html#DataTypeAnything)
/// represented as `Cow<'_, str>`.
///
/// - [SVG 1.1: Graphics Events](https://www.w3.org/TR/SVG11/script.html#GraphicsEvents)
/// - [SVG 1.1: SVG Events](https://www.w3.org/TR/SVG11/interact.html#SVGEvents)
#[derive(Debug, Clone, Default, BundleAttributes)]
pub struct GraphicalEvents<'a> {
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/interact.html#FocusInEvent)
    #[xml_attribute]
    pub onfocusin: Option<Cow<'a, str>>,
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/interact.html#FocusOutEvent)
    #[xml_attribute]
    pub onfocusout: Option<Cow<'a, str>>,
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/interact.html#ActivateEvent)
    #[xml_attribute]
    pub onactivate: Option<Cow<'a, str>>,
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/interact.html#ClickEvent)
    #[xml_attribute]
    pub onclick: Option<Cow<'a, str>>,
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/interact.html#MouseDownEvent)
    #[xml_attribute]
    pub onmousedown: Option<Cow<'a, str>>,
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/interact.html#MouseUpEvent)
    #[xml_attribute]
    pub onmouseup: Option<Cow<'a, str>>,
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/interact.html#MouseOverEvent)
    #[xml_attribute]
    pub onmouseover: Option<Cow<'a, str>>,
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/interact.html#MouseMoveEvent)
    #[xml_attribute]
    pub onmousemove: Option<Cow<'a, str>>,
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/interact.html#MouseOutEvent)
    #[xml_attribute]
    pub onmouseout: Option<Cow<'a, str>>,
}
