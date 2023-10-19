use ordered_float::OrderedFloat;

/// Floating point number representation re-exported to support precision
/// switching.
pub type Number = f32;

#[derive(Debug, Default, Clone, Copy)]
pub struct PositiveNumber {
    inner: Number,
}

impl PositiveNumber {
    pub const ZERO: PositiveNumber = PositiveNumber { inner: 0.0 };

    #[inline(always)]
    pub(crate) fn is_valid(value: Number) -> bool {
        !(value.is_nan() || value.is_infinite() || value.is_sign_negative())
    }

    pub fn new(value: Number) -> Option<Self> {
        if Self::is_valid(value) {
            Some(PositiveNumber { inner: value })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn new_unchecked(value: Number) -> Self {
        PositiveNumber { inner: value }
    }

    #[inline]
    pub fn to_inner(&self) -> Number {
        self.inner
    }

    #[inline]
    pub fn into_inner(self) -> Number {
        self.inner
    }
}

impl PartialEq for PositiveNumber {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        OrderedFloat(self.inner).eq(&OrderedFloat(other.inner))
    }
}
impl PartialEq<Number> for PositiveNumber {
    fn eq(&self, other: &Number) -> bool {
        if !PositiveNumber::is_valid(*other) {
            return false;
        }
        OrderedFloat(self.inner).eq(&OrderedFloat(*other))
    }
}
impl Eq for PositiveNumber {}

impl PartialOrd for PositiveNumber {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        OrderedFloat(self.inner).partial_cmp(&OrderedFloat(other.inner))
    }
}
impl Ord for PositiveNumber {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        OrderedFloat(self.inner).cmp(&OrderedFloat(other.inner))
    }
}

impl std::ops::Deref for PositiveNumber {
    type Target = Number;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for PositiveNumber {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(feature = "write")]
impl crate::io::Writable for PositiveNumber {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &crate::io::WriteSettings,
    ) -> std::io::Result<()> {
        write!(writer, "{:.prec$}", self.inner, prec = settings.precision)
    }
}

/// Unit identifiers.
///
/// Value must be one of the following:
/// "em", "ex", "px", "in", "cm", "mm", "pt", "pc".
/// 
/// In style sheets it can be either lower or uppercase, in presentation
/// attributes it must be lowercase. This crate will always generate a lowercase
/// presentation attribute value, even if parsed input file was uppercase.
///
/// [CSS2 specification](http://www.w3.org/TR/2008/REC-CSS2-20080411/syndata.html#length-units)
pub enum Unit {

}
