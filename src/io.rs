use std::borrow::Cow;

#[cfg(feature = "write")]
#[derive(Debug, Clone)]
pub struct WriteSettings {
    pub precision: usize,
}

#[cfg(feature = "write")]
impl Default for WriteSettings {
    fn default() -> Self {
        WriteSettings { precision: 4 }
    }
}

/// Unifies writing behavior between different types so their implementations
/// are easier to generate with the macro.
#[cfg(feature = "write")]
pub trait Writable {
    /// Writes this value to a writer.
    ///
    /// Written bytes must represent valid UTF-8 content to be stored in the
    /// document.
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &WriteSettings,
    ) -> std::io::Result<()>;

    fn write_to_string(&self, settings: &WriteSettings) -> String {
        let mut cursor = std::io::Cursor::new(Vec::new());
        self.write_to(&mut cursor, settings)
            .expect("unable to write to string buffer");
        unsafe {
            // SAFETY: write_to must only output valid UTF-8
            std::str::from_utf8_unchecked(cursor.into_inner().as_slice()).to_string()
        }
    }
}

/// Implementation of `From<String>` which is only called when a provided
/// `String` is known to be valid representation of constructed struct.
///
/// [`FromStr`](std::str::FromStr) should be used when validity of the passed
/// string isn't known.
pub trait FromStringUnsafe {
    unsafe fn from(value: String) -> Self;
}

impl<F: From<String>> FromStringUnsafe for F {
    unsafe fn from(value: String) -> Self {
        From::from(value)
    }
}

/// Type is a valid SVG value.
pub trait AttributeValue: ToString + FromStringUnsafe {
    #[cfg(feature = "write")]
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        _settings: &WriteSettings,
    ) -> std::io::Result<()>;

    /// Returns attribute value if it's a wrapper around a `AsRef<str>` type,
    /// i.e. backed by a `String` or `Cow<'_, str>`. `None` is returned when a
    /// `ToString` conversion is needed to acquire a string representation of
    /// the value.
    #[inline]
    fn as_str(&self) -> Option<&str> {
        return None;
    }
}

impl AttributeValue for Cow<'_, str> {
    #[cfg(feature = "write")]
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        _settings: &WriteSettings,
    ) -> std::io::Result<()> {
        writer.write(self.as_bytes())?;
        Ok(())
    }

    fn as_str(&self) -> Option<&str> {
        Some(self.as_ref())
    }
}

impl AttributeValue for String {
    #[cfg(feature = "write")]
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        _settings: &WriteSettings,
    ) -> std::io::Result<()> {
        writer.write(self.as_bytes())?;
        Ok(())
    }
}

#[cfg(feature = "write")]
impl<V: AttributeValue> Writable for V {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &WriteSettings,
    ) -> std::io::Result<()> {
        AttributeValue::write_to(self, writer, settings)
    }
}

/// Implemented by structs that represent context independant (named)
/// attributes.
pub trait Attribute<'a> {
    /// Attribute value type.
    type Value: AttributeValue;

    #[cfg(feature = "write")]
    fn write_attribute<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &WriteSettings,
    ) -> std::io::Result<()>;

    /// Returns the name of the attribute.
    fn name(&'a self) -> &'a str;

    /// Returns an immutable reference to this attribute's value.
    fn value(&self) -> &Self::Value;

    /// Returns an mutable reference to this attribute's value.
    fn value_mut(&mut self) -> &mut Self::Value;
}

/// Represents one or more **named** attributes.
///
/// This trait is implemented by both context independant attributes
/// such as [`DataAttribute`](crate::common::DataAttribute) (automatically) and
/// by structs grouping several attributes together.
///
/// This is also implemented on all tags to handle attribute generation
/// automatically.
///
/// Invoked by `#[xml_attribute_bundle]` field annotation.
pub trait AttributeBundle {
    #[cfg(feature = "write")]
    fn write_attributes<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &WriteSettings,
    ) -> std::io::Result<bool>;
}

impl<'a, A: Attribute<'a>> AttributeBundle for A {
    #[cfg(feature = "write")]
    fn write_attributes<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &WriteSettings,
    ) -> std::io::Result<bool> {
        self.write_attribute(writer, settings)?;
        Ok(true)
    }
}

impl<'a, A: Attribute<'a>> AttributeBundle for Option<A> {
    #[cfg(feature = "write")]
    fn write_attributes<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &WriteSettings,
    ) -> std::io::Result<bool> {
        match self {
            Some(it) => {
                it.write_attribute(writer, settings)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

impl<'a, A: Attribute<'a>> AttributeBundle for Vec<A> {
    #[cfg(feature = "write")]
    fn write_attributes<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &WriteSettings,
    ) -> std::io::Result<bool> {
        let mut any = false;
        for attrib in self {
            attrib.write_attribute(writer, settings)?;
            any = true;
        }
        Ok(any)
    }
}
