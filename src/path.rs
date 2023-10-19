use crate::{
    common::{ConditionalProcessing, CoreAttributes},
    script::GraphicalEvents,
};
use structuredvg_macros::BundleAttributes;

use crate::math::PositiveNumber;

#[cfg(feature = "path")]
mod path_impl {
    use crate::math::Number;

    /// Represents command types of [`CommandData`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    pub enum Command {
        Move,
        Line,
        Horizontal,
        Vertical,
        Cubic,
        CubicSmooth,
        Quadratic,
        QuadraticSmooth,
        Elliptical,
        Close,
    }

    impl Command {
        pub const fn argument_count(&self) -> usize {
            match self {
                Command::Move => 2,
                Command::Line => 2,
                Command::Horizontal => 1,
                Command::Vertical => 1,
                Command::Cubic => 6,
                Command::CubicSmooth => 4,
                Command::Quadratic => 4,
                Command::QuadraticSmooth => 2,
                Command::Elliptical => 7,
                Command::Close => 0,
            }
        }

        pub const fn absolute(&self) -> char {
            match self {
                Command::Move => 'M',
                Command::Line => 'L',
                Command::Horizontal => 'H',
                Command::Vertical => 'V',
                Command::Cubic => 'C',
                Command::CubicSmooth => 'S',
                Command::Quadratic => 'Q',
                Command::QuadraticSmooth => 'T',
                Command::Elliptical => 'A',
                Command::Close => 'z',
            }
        }

        pub const fn relative(&self) -> char {
            match self {
                Command::Move => 'm',
                Command::Line => 'l',
                Command::Horizontal => 'h',
                Command::Vertical => 'v',
                Command::Cubic => 'c',
                Command::CubicSmooth => 's',
                Command::Quadratic => 'q',
                Command::QuadraticSmooth => 't',
                Command::Elliptical => 'a',
                Command::Close => 'z',
            }
        }
    }

    /// a path segment command containing required parameters.
    ///
    /// See [SVG 1.1](https://www.w3.org/TR/SVG11/paths.html#PathData) and
    /// [SVG 2](https://www.w3.org/TR/SVG/paths.html#PathData) documentation for
    /// details on what each command does
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum CommandData {
        /// Move position without drawing any lines.
        ///
        /// When drawn, a fill will treat this segment like a solid line while
        /// stroke will skip it.
        Move([Number; 2]),
        /// Line segment.
        Line([Number; 2]),
        /// Horizontal line segment.
        Horizontal([Number; 1]),
        /// Vertical line segment.
        Vertical([Number; 1]),
        /// Cubic Bézier curve segment.
        Cubic([Number; 6]),
        /// Cubic Bézier curve segment.
        ///
        /// The first control point is assumed to be the reflection of the second
        /// control point on the previous command relative to the current point.
        CubicSmooth([Number; 4]),
        /// Quadratic Bézier curve segment.
        Quadratic([Number; 4]),
        /// Quadratic Bézier curve segment.
        ///
        /// The control point is assumed to be the reflection of the control point
        /// on the previous command relative to the current point.
        QuadraticSmooth([Number; 2]),
        /// Elliptical arc segment.
        Elliptical([Number; 7]),
        /// Line segment from current position to the beginning of the path.
        Close([Number; 0]),
    }

    impl CommandData {
        pub fn command(&self) -> Command {
            match self {
                CommandData::Move(_) => Command::Move,
                CommandData::Line(_) => Command::Line,
                CommandData::Horizontal(_) => Command::Horizontal,
                CommandData::Vertical(_) => Command::Vertical,
                CommandData::Cubic(_) => Command::Cubic,
                CommandData::CubicSmooth(_) => Command::CubicSmooth,
                CommandData::Quadratic(_) => Command::Quadratic,
                CommandData::QuadraticSmooth(_) => Command::QuadraticSmooth,
                CommandData::Elliptical(_) => Command::Elliptical,
                CommandData::Close(_) => Command::Close,
            }
        }

        pub fn args(&self) -> &[Number] {
            match self {
                CommandData::Move(args) => &args[..],
                CommandData::Line(args) => &args[..],
                CommandData::Horizontal(args) => &args[..],
                CommandData::Vertical(args) => &args[..],
                CommandData::Cubic(args) => &args[..],
                CommandData::CubicSmooth(args) => &args[..],
                CommandData::Quadratic(args) => &args[..],
                CommandData::QuadraticSmooth(args) => &args[..],
                CommandData::Elliptical(args) => &args[..],
                CommandData::Close(args) => &args[..],
            }
        }

        pub fn len(&self) -> usize {
            self.command().argument_count()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct PathSegment {
        pub relative: bool,
        pub data: CommandData,
    }

    #[cfg(feature = "write")]
    impl crate::io::Writable for PathSegment {
        fn write_to<W: std::io::Write>(
            &self,
            writer: &mut W,
            settings: &crate::io::WriteSettings,
        ) -> std::io::Result<()> {
            if self.relative {
                writer.write(&[self.data.command().relative() as u8])?;
            } else {
                writer.write(&[self.data.command().absolute() as u8])?;
            }

            match self.data {
                CommandData::Horizontal(it) | CommandData::Vertical(it) => {
                    writer.write_fmt(format_args!(
                        "{:.prec$}",
                        it[0],
                        prec = settings.precision
                    ))?;
                }
                CommandData::Move(it)
                | CommandData::Line(it)
                | CommandData::QuadraticSmooth(it) => {
                    writer.write_fmt(format_args!(
                        "{:.prec$} {:.prec$}",
                        it[0],
                        it[1],
                        prec = settings.precision
                    ))?;
                }
                CommandData::CubicSmooth(it) | CommandData::Quadratic(it) => {
                    writer.write_fmt(format_args!(
                        "{:.prec$} {:.prec$} {:.prec$} {:.prec$}",
                        it[0],
                        it[1],
                        it[2],
                        it[3],
                        prec = settings.precision
                    ))?;
                }
                CommandData::Cubic(it) => {
                    writer.write_fmt(format_args!(
                        "{:.prec$} {:.prec$} {:.prec$} {:.prec$} {:.prec$} {:.prec$}",
                        it[0],
                        it[1],
                        it[2],
                        it[3],
                        it[4],
                        it[5],
                        prec = settings.precision
                    ))?;
                }
                CommandData::Elliptical(it) => {
                    writer.write_fmt(format_args!(
                        "{:.prec$} {:.prec$} {:.prec$} {:.prec$} {:.prec$} {:.prec$} {:.prec$}",
                        it[0],
                        it[1],
                        it[2],
                        it[3],
                        it[4],
                        it[5],
                        it[6],
                        prec = settings.precision
                    ))?;
                }
                CommandData::Close(_) => {}
            }

            Ok(())
        }
    }

    /// Type safe representation of path data.
    ///
    /// See [SVG 1.1](https://www.w3.org/TR/SVG11/paths.html#PathData) and
    /// [SVG 2](https://www.w3.org/TR/SVG/paths.html#PathData) documentation for
    /// more details.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PathData {
        pub segments: Vec<PathSegment>,
    }

    #[cfg(feature = "write")]
    impl crate::io::Writable for PathData {
        fn write_to<W: std::io::Write>(
            &self,
            writer: &mut W,
            settings: &crate::io::WriteSettings,
        ) -> std::io::Result<()> {
            for segment in &self.segments {
                segment.write_to(writer, settings)?;
            }
            Ok(())
        }
    }
}
#[cfg(feature = "path")]
pub use path_impl::*;

#[cfg(feature = "path")]
type PathDataImpl<'a> = path_impl::PathData;
#[cfg(not(feature = "path"))]
type PathDataImpl<'a> = std::borrow::Cow<'a, str>;

#[derive(Debug, Clone, BundleAttributes)]
pub struct ElementPath<'a> {
    /// Conditional processing attributes.
    #[xml_attribute_bundle]
    pub conditional_processing: Box<ConditionalProcessing<'a>>,

    /// Core attributes.
    #[xml_attribute_bundle]
    pub core: Box<CoreAttributes<'a>>,

    /// Graphical event attributes.
    #[xml_attribute_bundle]
    pub graphical_event: Box<GraphicalEvents<'a>>,

    /// Specifies shape of the path.
    ///
    /// - [SVG 1.1 Documentation](https://www.w3.org/TR/SVG11/paths.html#DAttribute)
    /// - [SVG 2 Documentation](https://www.w3.org/TR/SVG/paths.html#DProperty)
    #[xml_attribute]
    pub d: Option<PathDataImpl<'a>>,

    /// Author's computation of the total length of the path, in user units.
    ///
    /// - [SVG 1.1 Documentation](https://www.w3.org/TR/SVG11/paths.html#PathLengthAttribute)
    /// - [SVG 2 Documentation](https://www.w3.org/TR/SVG/paths.html#PathLengthAttribute)
    #[xml_attribute]
    pub path_length: Option<PositiveNumber>,
}

#[cfg(feature = "write")]
impl crate::io::Writable for ElementPath<'_> {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &crate::io::WriteSettings,
    ) -> std::io::Result<()> {
        writer.write(b"<path ")?;
        crate::io::AttributeBundle::write_attributes(self, writer, settings)?;
        writer.write(b"/>")?;
        Ok(())
    }
}
