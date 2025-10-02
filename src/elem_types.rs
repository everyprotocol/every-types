use core::convert::TryFrom;
use derive_more::Display;

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum ElementTypeError {
	#[display("unknown discriminant: {0:#04x}")]
	UnknownDiscriminant(u8),
	#[display("{0} is not a MatterForm")]
	NotAMatterForm(ElementType),
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
#[repr(u8)]
pub enum ElementType {
	// Simple
	Json = 0x01,
	Image = 0x02,
	// Code
	Wasm = 0xC0,
	// Data Collection
	Enum = 0xD0,
	Perm = 0xD1,
	// Meta objects
	Set = 0xE1,
	Kind = 0xE2,
	Relation = 0xE3,
	Value = 0xE4,
	Unique = 0xE5,
	// Plain object
	Plain = 0xFE,
	// Information
	Info = 0xFF,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
#[repr(u8)]
pub enum MatterForm {
	// Simple
	Json = 0x01,
	Image = 0x02,
	// Code
	Wasm = 0xC0,
	// Data Collection
	Enum = 0xD0,
	Perm = 0xD1,
}

impl TryFrom<u8> for ElementType {
	type Error = ElementTypeError;
	fn try_from(v: u8) -> Result<Self, Self::Error> {
		Ok(match v {
			0x01 => ElementType::Json,
			0x02 => ElementType::Image,
			0xC0 => ElementType::Wasm,
			0xD0 => ElementType::Enum,
			0xD1 => ElementType::Perm,
			0xE1 => ElementType::Set,
			0xE2 => ElementType::Kind,
			0xE3 => ElementType::Relation,
			0xE4 => ElementType::Value,
			0xE5 => ElementType::Unique,
			0xFE => ElementType::Plain,
			0xFF => ElementType::Info,
			_ => return Err(ElementTypeError::UnknownDiscriminant(v)),
		})
	}
}

impl From<ElementType> for u8 {
	fn from(e: ElementType) -> Self {
		e as u8
	}
}

impl TryFrom<u8> for MatterForm {
	type Error = ElementTypeError;
	fn try_from(v: u8) -> Result<Self, Self::Error> {
		Ok(match v {
			0x01 => MatterForm::Json,
			0x02 => MatterForm::Image,
			0xC0 => MatterForm::Wasm,
			0xD0 => MatterForm::Enum,
			0xD1 => MatterForm::Perm,
			_ => return Err(ElementTypeError::UnknownDiscriminant(v)),
		})
	}
}

impl From<MatterForm> for u8 {
	fn from(f: MatterForm) -> Self {
		f as u8
	}
}

impl From<MatterForm> for ElementType {
	fn from(f: MatterForm) -> Self {
		match f {
			MatterForm::Json => ElementType::Json,
			MatterForm::Image => ElementType::Image,
			MatterForm::Wasm => ElementType::Wasm,
			MatterForm::Enum => ElementType::Enum,
			MatterForm::Perm => ElementType::Perm,
		}
	}
}

impl TryFrom<ElementType> for MatterForm {
	type Error = ElementTypeError;
	fn try_from(e: ElementType) -> Result<Self, Self::Error> {
		match e {
			ElementType::Json => Ok(MatterForm::Json),
			ElementType::Image => Ok(MatterForm::Image),
			ElementType::Wasm => Ok(MatterForm::Wasm),
			ElementType::Enum => Ok(MatterForm::Enum),
			ElementType::Perm => Ok(MatterForm::Perm),
			other => Err(ElementTypeError::NotAMatterForm(other)),
		}
	}
}
