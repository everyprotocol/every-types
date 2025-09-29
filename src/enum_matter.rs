use crate::{Result, SimpleMatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumMatterError {
	TooShort,
	BadMagic([u8; 4]),
	BadVersion(u8),
	TooManyCols(u8),
	ZeroDims,
	BadElementType { index: usize, emty: u8 },
	BodyTooShort { need: usize, got: usize },
	BodyTooLong { expect: usize, got: usize },
	OobCell { row: usize, col: usize },
	Overflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumMatterHeader {
	pub magic: [u8; 4],
	pub ver: u8,
	pub cols: u8,
	pub rows: u16,
	pub _unused: u64,
	pub emtys: [u8; 16],
}

impl EnumMatterHeader {
	pub const MAGIC: [u8; 4] = *b"ENUM";
	pub const CELL_SIZE: usize = 32;
	pub const HEADER_SIZE: usize = 32;

	pub fn from(blob: &[u8]) -> Result<Self, EnumMatterError> {
		if blob.len() < Self::HEADER_SIZE {
			return Err(EnumMatterError::TooShort);
		}

		let magic: [u8; 4] = blob[0..4].try_into().unwrap();
		if magic != Self::MAGIC {
			return Err(EnumMatterError::BadMagic(magic));
		}

		let ver = blob[4];
		if ver != 1 {
			return Err(EnumMatterError::BadVersion(ver));
		}

		let cols = blob[5];
		if cols == 0 {
			return Err(EnumMatterError::ZeroDims);
		}
		if cols > 16 {
			return Err(EnumMatterError::TooManyCols(cols));
		}

		let rows = u16::from_le_bytes(blob[6..8].try_into().unwrap());
		if rows == 0 {
			return Err(EnumMatterError::ZeroDims);
		}

		let _unused = u64::from_le_bytes(blob[8..16].try_into().unwrap());
		let emtys: [u8; 16] = blob[16..32].try_into().unwrap();

		// for the first `cols`, each must be Info(0xFF) or a SimpleMatter.
		for (i, &b) in emtys.iter().take(cols as usize).enumerate() {
			if b != 0xFF && !SimpleMatter::test(b) {
				return Err(EnumMatterError::BadElementType { index: i, emty: b });
			}
		}

		Ok(Self { magic, ver, cols, rows, _unused, emtys })
	}

	#[inline]
	pub fn cols(&self) -> usize {
		self.cols as usize
	}

	#[inline]
	pub fn rows(&self) -> usize {
		self.rows as usize
	}
}

#[derive(Debug, Clone)]
pub struct EnumMatter {
	pub header: EnumMatterHeader,
	pub body: Vec<u8>, // len = cols * rows * 32
}

impl EnumMatter {
	pub fn from(blob: &[u8]) -> Result<Self, EnumMatterError> {
		let header = EnumMatterHeader::from(blob)?;
		let cols = header.cols();
		let rows = header.rows();

		let cells = cols.checked_mul(rows).ok_or(EnumMatterError::Overflow)?;
		let expect_len = cells
			.checked_mul(EnumMatterHeader::CELL_SIZE)
			.ok_or(EnumMatterError::Overflow)?;

		let got_len = blob
			.len()
			.checked_sub(EnumMatterHeader::HEADER_SIZE)
			.ok_or(EnumMatterError::TooShort)?;
		if got_len < expect_len {
			return Err(EnumMatterError::BodyTooShort { need: expect_len, got: got_len });
		}
		if got_len > expect_len {
			return Err(EnumMatterError::BodyTooLong { expect: expect_len, got: got_len });
		}

		let body = blob[EnumMatterHeader::HEADER_SIZE..EnumMatterHeader::HEADER_SIZE + expect_len]
			.to_vec();
		Ok(Self { header, body })
	}

	#[inline]
	pub fn cols(&self) -> usize {
		self.header.cols()
	}

	#[inline]
	pub fn rows(&self) -> usize {
		self.header.rows()
	}

	/// Returns a 32-byte cell at (row, col), 0-based.
	pub fn cell(&self, row: usize, col: usize) -> Result<&[u8; 32], EnumMatterError> {
		let rows = self.rows();
		let cols = self.cols();
		if row >= rows || col >= cols {
			return Err(EnumMatterError::OobCell { row, col });
		}

		let idx = row
			.checked_mul(cols)
			.ok_or(EnumMatterError::Overflow)?
			.checked_add(col)
			.ok_or(EnumMatterError::Overflow)?;
		let offset =
			idx.checked_mul(EnumMatterHeader::CELL_SIZE).ok_or(EnumMatterError::Overflow)?;
		let end = offset + EnumMatterHeader::CELL_SIZE;

		let cell: &[u8; 32] = self.body[offset..end].try_into().unwrap();
		Ok(cell)
	}
}
