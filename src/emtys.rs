use derive_more::Display;

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
#[repr(u8)]
pub enum ElementType {
	// Simple Matter
	Json = 0x01,
	Image = 0x02,

	// Complex Matter
	Enum = 0xE0,
	Perm = 0xE1,

	// Meta Object
	Set = 0xF1,
	Kind = 0xF2,
	Relation = 0xF3,
	Value = 0xF4,
	Unique = 0xF5,

	// Plain Object
	Plain = 0xFE,

	// Information
	Info = 0xFF,
}

impl ElementType {
	pub fn is_simple_matter(self) -> bool {
		SimpleMatter::test(self as u8)
	}

	pub fn is_complex_matter(self) -> bool {
		ComplexMatter::test(self as u8)
	}

	pub fn is_matter(self) -> bool {
		(0x01u8..=0xEF).contains(&(self as u8))
	}

	pub fn is_meta_object(self) -> bool {
		MetaObject::test(self as u8)
	}

	pub fn is_plain_object(self) -> bool {
		0xFE == (self as u8)
	}

	pub fn is_object(self) -> bool {
		(0xF0u8..=0xFE).contains(&(self as u8))
	}

	pub fn is_info(self) -> bool {
		0xFF == (self as u8)
	}
}

impl From<ElementType> for u8 {
	fn from(e: ElementType) -> u8 {
		e as u8
	}
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
#[repr(u8)]
pub enum SimpleMatter {
	Json = 0x01,
	Image = 0x02,
}

impl SimpleMatter {
	pub fn test(e: u8) -> bool {
		(0x01u8..=0xDF).contains(&e)
	}
}

impl From<SimpleMatter> for u8 {
	fn from(e: SimpleMatter) -> u8 {
		e as u8
	}
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
#[repr(u8)]
pub enum ComplexMatter {
	Enum = 0xE0,
	Perm = 0xE1,
}

impl ComplexMatter {
	pub fn test(e: u8) -> bool {
		(0xE0u8..=0xEF).contains(&e)
	}
}

impl From<ComplexMatter> for u8 {
	fn from(e: ComplexMatter) -> u8 {
		e as u8
	}
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
#[repr(u8)]
pub enum MetaObject {
	Set = 0xF1,
	Kind = 0xF2,
	Relation = 0xF3,
	Value = 0xF4,
	Unique = 0xF5,
}

impl MetaObject {
	pub fn test(e: u8) -> bool {
		(0xF0u8..=0xFD).contains(&e)
	}
}

impl From<MetaObject> for u8 {
	fn from(e: MetaObject) -> u8 {
		e as u8
	}
}

impl From<SimpleMatter> for ElementType {
	fn from(s: SimpleMatter) -> Self {
		match s {
			SimpleMatter::Json => ElementType::Json,
			SimpleMatter::Image => ElementType::Image,
		}
	}
}

impl From<ComplexMatter> for ElementType {
	fn from(c: ComplexMatter) -> Self {
		match c {
			ComplexMatter::Enum => ElementType::Enum,
			ComplexMatter::Perm => ElementType::Perm,
		}
	}
}

impl From<MetaObject> for ElementType {
	fn from(k: MetaObject) -> Self {
		match k {
			MetaObject::Set => ElementType::Set,
			MetaObject::Kind => ElementType::Kind,
			MetaObject::Relation => ElementType::Relation,
			MetaObject::Value => ElementType::Value,
			MetaObject::Unique => ElementType::Unique,
		}
	}
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum ElementTypeError {
	UnknownElementType(u8),
	UnknownSimpleMatter(u8),
	UnknownComplexMatter(u8),
	UnknownObjectKind(u8),
}

impl TryFrom<u8> for ElementType {
	type Error = ElementTypeError;
	fn try_from(x: u8) -> Result<Self, Self::Error> {
		Ok(match x {
			0x01 => ElementType::Json,
			0x02 => ElementType::Image,
			0xE0 => ElementType::Enum,
			0xE1 => ElementType::Perm,
			0xF1 => ElementType::Set,
			0xF2 => ElementType::Kind,
			0xF3 => ElementType::Relation,
			0xF4 => ElementType::Value,
			0xF5 => ElementType::Unique,
			0xFE => ElementType::Plain,
			0xFF => ElementType::Info,
			_ => return Err(ElementTypeError::UnknownElementType(x)),
		})
	}
}

impl TryFrom<u8> for SimpleMatter {
	type Error = ElementTypeError;
	fn try_from(x: u8) -> Result<Self, Self::Error> {
		Ok(match x {
			0x01 => SimpleMatter::Json,
			0x02 => SimpleMatter::Image,
			_ => return Err(ElementTypeError::UnknownSimpleMatter(x)),
		})
	}
}

impl TryFrom<u8> for ComplexMatter {
	type Error = ElementTypeError;
	fn try_from(x: u8) -> Result<Self, Self::Error> {
		Ok(match x {
			0xE0 => ComplexMatter::Enum,
			0xE1 => ComplexMatter::Perm,
			_ => return Err(ElementTypeError::UnknownComplexMatter(x)),
		})
	}
}

impl TryFrom<u8> for MetaObject {
	type Error = ElementTypeError;
	fn try_from(x: u8) -> Result<Self, Self::Error> {
		Ok(match x {
			0xF1 => MetaObject::Set,
			0xF2 => MetaObject::Kind,
			0xF3 => MetaObject::Relation,
			0xF4 => MetaObject::Value,
			0xF5 => MetaObject::Unique,
			_ => return Err(ElementTypeError::UnknownObjectKind(x)),
		})
	}
}

impl TryFrom<ElementType> for SimpleMatter {
	type Error = ElementTypeError;
	fn try_from(e: ElementType) -> Result<Self, Self::Error> {
		SimpleMatter::try_from(u8::from(e))
	}
}

impl TryFrom<ElementType> for ComplexMatter {
	type Error = ElementTypeError;
	fn try_from(e: ElementType) -> Result<Self, Self::Error> {
		ComplexMatter::try_from(u8::from(e))
	}
}

impl TryFrom<ElementType> for MetaObject {
	type Error = ElementTypeError;
	fn try_from(e: ElementType) -> Result<Self, Self::Error> {
		MetaObject::try_from(u8::from(e))
	}
}
