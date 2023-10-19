use std::{borrow::Cow, fmt::Write, marker::PhantomData, str::FromStr};

use structuredvg_macros::BundleAttributes;

use crate::{error::InvalidLanguageTag, io::*, style::DeclarationList};

/// Represents a collection of values `V` stored as a `DELIMITER` separated list
/// in the document.
///
/// When writing, no whitespace will be emitted surrounding the delimiters, but
/// they are allowed and will be dropped when reading.
#[derive(Debug, Default, Clone, PartialEq)]
#[repr(transparent)]
pub struct DelimitedValues<const DELIMITER: char, V: AttributeValue = String> {
    inner: String,
    _phantom: PhantomData<V>,
}

impl<const DELIMITER: char, V: AttributeValue> DelimitedValues<DELIMITER, V> {
    #[inline]
    pub fn new() -> Self {
        DelimitedValues {
            inner: String::new(),
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        DelimitedValues {
            inner: String::with_capacity(capacity),
            _phantom: PhantomData,
        }
    }

    pub fn push(&mut self, value: V) {
        if !self.inner.is_empty() {
            self.inner
                .write_char(DELIMITER)
                .expect("unable to push delimiter");
        }

        match value.as_str() {
            Some(it) => self.inner.write_str(it),
            None => self.inner.write_str(value.to_string().as_str()),
        }
        .expect("unable to push value");
    }

    /// # Safety
    ///
    /// This method is safe if pushed `&str` is a valid textual representation
    /// of attribute value `V`.
    /// That means that if `FromStr` were implemented, `V::from_str` wouldn't
    /// return an error while parsing it.
    pub unsafe fn push_str(&mut self, value: &str) {
        if !self.inner.is_empty() {
            self.inner
                .write_char(DELIMITER)
                .expect("unable to push delimiter");
        }

        self.inner.write_str(value).expect("unable to push value");
    }

    // TODO: Track DelimitedValues indices?
    // would maybe speed up mutation at the cost of memory consumption?

    pub fn pop(&mut self) -> Option<V> {
        if let Some(last) = self.inner.rfind(DELIMITER) {
            let mut last = self.inner.drain(last..);
            let _ = last.next(); // drop delimiter
            Some(unsafe {
                // SAFETY: All values stored in the container come from
                // V::to_string()
                FromStringUnsafe::from(last.collect::<String>())
            })
        } else {
            None
        }
    }

    /// Removes `value` from this list or returns `false` if it's not present.
    pub fn remove(&mut self, value: &V) -> bool {
        let start = match value.as_str() {
            Some(it) => self.inner.find(it),
            None => self.inner.find(value.to_string().as_str()),
        };

        if let Some(mut start) = start {
            let mut end = start
                + value
                    .as_str()
                    .map(|it| it.len())
                    .unwrap_or_else(|| value.to_string().len());
            if end != self.inner.len() {
                // Not at the end
                end += 1;
            }
            if start != 0 {
                // Not at the beginning
                start -= 1;
            }
            self.inner.drain(start..end).count();
            true
        } else {
            false
        }
    }

    pub fn contains(&mut self, value: &V) -> bool {
        let position = match value.as_str() {
            Some(it) => self.inner.find(it),
            None => self.inner.find(value.to_string().as_str()),
        };

        position.is_some()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        self.inner.split(DELIMITER)
    }

    #[inline]
    pub fn iter_values(&self) -> impl Iterator<Item = V> + '_ {
        self.inner.split(DELIMITER).map(|it| unsafe {
            // SAFETY: All values stored in the container come from
            // V::to_string()
            FromStringUnsafe::from(it.to_string())
        })
    }
}

impl<const DELIMITER: char, V: AttributeValue> AsRef<str> for DelimitedValues<DELIMITER, V> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl<const DELIMITER: char, V: AttributeValue> ToString for DelimitedValues<DELIMITER, V> {
    fn to_string(&self) -> String {
        self.inner.clone()
    }
}

#[cfg(feature = "write")]
impl<const DELIMITER: char, V: AttributeValue> crate::io::Writable
    for DelimitedValues<DELIMITER, V>
{
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        _settings: &crate::io::WriteSettings,
    ) -> std::io::Result<()> {
        writer.write(self.as_ref().as_bytes())?;
        Ok(())
    }
}

/// `xml:space` value that specifies whether white space is preserved in
/// character data.
///
/// For details see
/// [White space handling](https://www.w3.org/TR/SVG11/text.html#WhiteSpace)
/// section of the specification.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum XmlSpace {
    #[default]
    Default,
    Preserve,
}

/// Type safe representation of a language tag.
///
/// Value should follow [RFC 5646](https://www.rfc-editor.org/info/rfc5646).
///
/// While this isn't checked for performance reasons, using non-standard names
/// will cause the attribute to be ignored by most software relying on the
/// value. That can cause further issues with localization and screen readers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageTag<'a>(Cow<'a, str>);

impl<'a> LanguageTag<'a> {
    /// Constructs a new language tag.
    ///
    /// Value should follow [RFC 5646](https://www.rfc-editor.org/info/rfc5646).
    ///
    /// An error is never thrown but it's there for semantic reasons (currently),
    /// and to provide version safety if the crate ever starts checking the value.
    #[inline]
    pub fn new(value: impl Into<Cow<'a, str>>) -> Result<Self, InvalidLanguageTag> {
        Ok(LanguageTag(value.into()))
    }
}

impl ToString for LanguageTag<'_> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for LanguageTag<'_> {
    type Err = InvalidLanguageTag;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(LanguageTag(Cow::Owned(s.to_string())))
    }
}

impl FromStringUnsafe for LanguageTag<'_> {
    unsafe fn from(value: String) -> Self {
        LanguageTag(Cow::Owned(value))
    }
}

impl<'a> AttributeValue for LanguageTag<'a> {
    #[cfg(feature = "write")]
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        _settings: &WriteSettings,
    ) -> std::io::Result<()> {
        writer.write(self.0.as_bytes())?;
        Ok(())
    }

    fn as_str(&self) -> Option<&str> {
        return None;
    }
}

impl std::ops::Deref for LanguageTag<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// Base structure of all SVG elements used to compose common element attributes
/// onto all other elements provided by this crate.
///
/// Attributes provided on this struct should follow "Common attributes"
/// sections of [SVG 1.1](https://www.w3.org/TR/SVG11/intro.html#TermCoreAttributes)
/// specification.
#[derive(Debug, Clone, Default, BundleAttributes)]
pub struct CoreAttributes<'a> {
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/struct.html#IDAttribute)
    #[xml_attribute{
        transform: id.as_bytes()
    }]
    pub id: Option<Cow<'a, str>>,

    /// This attribute is part of SVG 2 specification, but it's part of
    /// [HTML5 standard](https://html.spec.whatwg.org/multipage/interaction.html#dom-tabindex)
    /// so it's included under `html` feature flag.
    ///
    /// [SVG 2 documentation](https://www.w3.org/TR/SVG/struct.html#tabindexattribute)
    #[cfg(feature = "html")]
    #[xml_attribute{
        transform: tabindex.to_string().as_bytes()
    }]
    pub tabindex: Option<isize>,

    /// Specifies the primary language for the element's contents and for any of
    /// the element's attributes that contain text.
    ///
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/struct.html#XMLLangAttribute)
    #[xml_attribute{
        name: "xml:lang",
        transform: xml_lang.as_bytes()
    }]
    pub xml_lang: Option<LanguageTag<'a>>,

    /// Standard XML attribute to specify whether white space is preserved in
    /// character data.
    ///
    /// Note that this attribute is removed is deprecated in SVG 2 specification
    /// in favor of [`white-space`](https://www.w3.org/TR/css-text-3/#white-space-property).
    ///
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/struct.html#XMLSpaceAttribute)
    #[cfg(feature = "html")]
    #[xml_attribute{
        name: "xml:space",
        check: Default,
        literal: b"preserve"
    }]
    pub xml_space: XmlSpace,

    /// Class names of the element.
    ///
    /// This attribute is part of SVG 2 specification, but HTML supports it on
    /// any element so it's provided through `html` feature flag.
    ///
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/styling.html#ClassAttribute)
    #[xml_attribute]
    pub class: Option<DelimitedValues<' '>>,
    /// Custom per-element style rules.
    ///
    /// This attribute is part of SVG 2 specification, but HTML supports it on
    /// any element so it's provided through `html` feature flag.
    ///
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/styling.html#StyleAttribute)
    #[xml_attribute]
    pub style: Option<DeclarationList<'a>>,

    /// Custom data attributes.
    ///
    /// This attribute is part of SVG 2 specification, but HTML supports it on
    /// any element so it's provided through `html` feature flag.
    ///
    /// [SVG 2 documentation](https://www.w3.org/TR/SVG/struct.html#DataAttributes)
    #[cfg(feature = "html")]
    #[xml_attribute_bundle]
    pub data: Vec<DataAttribute<'a>>,

    /// Attributes that aren't specified by the [standard](https://www.w3.org/TR/SVG11)
    /// or implemented.
    ///
    /// All [styling properties](https://www.w3.org/TR/SVG11/styling.html#SVGStylingProperties)
    /// are located here as well as any non-standard ones.
    #[xml_attribute_bundle]
    pub other: Vec<NonStandardAttribute<'a>>,
}

/// Represents a `data-*` attribute.
///
/// `name` should must be at least one character long, must be
/// [XML-compatible](https://www.w3.org/TR/2014/CR-html5-20140204/infrastructure.html#xml-compatible)
/// and shouldn't contain any uppercase ASCII letters.
///
/// For details see [HTML5 specification](https://www.w3.org/TR/2014/CR-html5-20140204/dom.html#embedding-custom-non-visible-data-with-the-data-*-attributes).
#[cfg(feature = "html")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataAttribute<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<'a> DataAttribute<'a> {
    /// Creates a new data-* attribute from provided `name` and `value`.
    ///
    /// `name` shouldn't contain a "data-" prefix as it's added by this
    ///constructor.
    pub fn new(name: impl AsRef<str>, value: impl Into<Cow<'a, str>>) -> Self {
        DataAttribute {
            name: Cow::Owned("data-".to_string() + name.as_ref()),
            value: value.into(),
        }
    }
}

#[cfg(feature = "html")]
impl<'a> Attribute<'a> for DataAttribute<'a> {
    type Value = Cow<'a, str>;

    #[cfg(feature = "write")]
    fn write_attribute<W: std::io::Write>(
        &self,
        writer: &mut W,
        _settings: &WriteSettings,
    ) -> std::io::Result<()> {
        write!(writer, "{}=\"{}\"", self.name, self.value)
    }

    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn value_mut(&mut self) -> &mut Self::Value {
        &mut self.value
    }
}

/// Contains a non-standard attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonStandardAttribute<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<'a> Attribute<'a> for NonStandardAttribute<'a> {
    type Value = Cow<'a, str>;

    #[cfg(feature = "write")]
    fn write_attribute<W: std::io::Write>(
        &self,
        writer: &mut W,
        _settings: &WriteSettings,
    ) -> std::io::Result<()> {
        write!(writer, "{}=\"{}\"", self.name, self.value)
    }

    fn name(&'a self) -> &'a str {
        &self.name
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn value_mut(&mut self) -> &mut Self::Value {
        &mut self.value
    }
}

/// These arguments provide an ability to specify alternate viewing depending on
/// the capabilities of a given user agent or the user's language.
///
/// For details see [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/struct.html#ConditionalProcessing).
#[derive(Debug, Clone, Default, BundleAttributes)]
pub struct ConditionalProcessing<'a> {
    /// List of required user agent features.
    ///
    /// For a list of allowed values see [Feature Strings](https://www.w3.org/TR/SVG11/feature.html)
    /// appendix.
    ///
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/struct.html#RequiredFeaturesAttribute)
    #[xml_attribute {
        name: "requiredFeatures",
    }]
    pub required_features: Option<DelimitedValues<' '>>,

    /// Defines a list of required language extensions declared through
    /// [IRI references](https://www.w3.org/TR/SVG11/linking.html#IRIReference).
    ///
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/struct.html#RequiredExtensionsAttribute)
    #[xml_attribute {
        name: "requiredExtensions",
    }]
    pub required_extensions: Option<DelimitedValues<' '>>,

    /// List of supported languages.
    ///
    /// [SVG 1.1 documentation](https://www.w3.org/TR/SVG11/struct.html#SystemLanguageAttribute)
    #[xml_attribute {
        name: "systemLanguage",
    }]
    pub system_language: Option<DelimitedValues<',', LanguageTag<'a>>>,
}
