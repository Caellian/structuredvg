use std::borrow::Cow;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DeclarationList<'a> {
    pub declarations: Vec<Declaration<'a>>,
}

impl<'a> DeclarationList<'a> {
    #[inline]
    pub fn push_property(&mut self, name: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) {
        self.declarations.push(Declaration::Property {
            name: name.into(),
            value: value.into(),
        })
    }
}

#[cfg(feature = "write")]
impl crate::io::Writable for DeclarationList<'_> {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        settings: &crate::io::WriteSettings,
    ) -> std::io::Result<()> {
        let non_empty: Vec<_> = self
            .declarations
            .iter()
            .filter(|it| !it.is_empty())
            .collect();
        if non_empty.len() > 0 {
            for declaration in non_empty.iter().take(self.declarations.len() - 1) {
                if declaration.is_empty() {
                    continue;
                }
                declaration.write_to(writer, settings)?;
                writer.write(b";")?;
            }
            self.declarations
                .last()
                .unwrap()
                .write_to(writer, settings)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Declaration<'a> {
    #[default]
    Empty,
    Property {
        name: Cow<'a, str>,
        value: Cow<'a, str>,
    },
}

impl<'a> Declaration<'a> {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

#[cfg(feature = "write")]
impl crate::io::Writable for Declaration<'_> {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        _settings: &crate::io::WriteSettings,
    ) -> std::io::Result<()> {
        match self {
            Self::Empty => Ok(()),
            Self::Property { name, value } => {
                writer.write(name.as_bytes())?;
                writer.write(b":")?;
                writer.write(value.as_bytes())?;
                Ok(())
            }
        }
    }
}
