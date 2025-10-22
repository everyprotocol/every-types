use crate::{Result, Vec};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum EnumMatterError {
	#[error("invalid EnumMatter header")]
	BadHeader,

	#[error("bad magic: expected 'ENUM' (45 4E 55 4D), got {0:02X?}")]
	BadMagic([u8; 4]),

	#[error("unsupported version {0} (expected 1)")]
	BadVersion(u8),

	#[error("aux count {0} exceeds maximum of 8")]
	BadAuxCount(u8),

	#[error("column count {0} exceeds maximum of 16")]
	BadColCount(u8),

	#[error("invalid aux types layout (first N must be >0, remaining must be 0)")]
	BadAuxTypes,

	#[error("invalid column types layout (first N must be >0, remaining must be 0)")]
	BadColTypes,

	#[error("invalid body length: expected {expect} bytes, got {got} bytes")]
	BadBody { expect: usize, got: usize },

	#[error("cell out of bounds at (row={row}, col={col})")]
	OobCell { row: usize, col: usize },

	#[error("aux index out of bounds: {index}")]
	OobAux { index: usize },

	#[error("arithmetic overflow")]
	Overflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumMatterHeader {
	pub magic: [u8; 4],      // "ENUM"
	pub ver_aux: u8,         // hi4 = version, lo4 = aux_count (0..=8)
	pub cols: u8,            // 0..=16
	pub rows: u16,           // >=0
	pub aux_types: [u8; 8],  // right-padded with 0
	pub col_types: [u8; 16], // right-padded with 0
}

impl EnumMatterHeader {
	pub const MAGIC: [u8; 4] = *b"ENUM";
	pub const CELL_SIZE: usize = 32;
	pub const HEADER_SIZE: usize = 32;

	#[inline]
	pub fn version(&self) -> u8 {
		self.ver_aux >> 4
	}

	#[inline]
	pub fn aux(&self) -> usize {
		(self.ver_aux & 0x0F) as usize
	}

	#[inline]
	pub fn cols(&self) -> usize {
		self.cols as usize
	}

	#[inline]
	pub fn rows(&self) -> usize {
		self.rows as usize
	}

	pub fn from(blob: &[u8]) -> Result<Self, EnumMatterError> {
		if blob.len() < Self::HEADER_SIZE {
			return Err(EnumMatterError::BadHeader);
		}

		let magic: [u8; 4] = blob[0..4].try_into().unwrap();
		if magic != Self::MAGIC {
			return Err(EnumMatterError::BadMagic(magic));
		}

		let ver_aux = blob[4];
		let ver = ver_aux >> 4;
		let aux = ver_aux & 0x0F;
		if ver != 1 {
			return Err(EnumMatterError::BadVersion(ver));
		}
		if aux > 8 {
			return Err(EnumMatterError::BadAuxCount(aux));
		}

		let cols = blob[5];
		if cols > 16 {
			return Err(EnumMatterError::BadColCount(cols));
		}

		let rows = u16::from_le_bytes(blob[6..8].try_into().unwrap());

		// Validate aux_types: first aux > 0, rest == 0
		let aux_types: [u8; 8] = blob[8..16].try_into().unwrap();
		{
			let (active, pad) = aux_types.split_at(aux as usize);
			if active.iter().any(|&t| t == 0) || pad.iter().any(|&t| t != 0) {
				return Err(EnumMatterError::BadAuxTypes);
			}
		}

		// Validate col_types: first cols > 0, rest == 0
		let col_types: [u8; 16] = blob[16..32].try_into().unwrap();
		{
			let (active, pad) = col_types.split_at(cols as usize);
			if active.iter().any(|&t| t == 0) || pad.iter().any(|&t| t != 0) {
				return Err(EnumMatterError::BadColTypes);
			}
		}

		Ok(Self { magic, ver_aux, cols, rows, aux_types, col_types })
	}
}

#[derive(Debug, Clone)]
pub struct EnumMatter {
	pub header: EnumMatterHeader,
	pub aux_data: Vec<u8>, // len = aux * 32
	pub row_data: Vec<u8>, // len = rows * cols * 32
}

impl EnumMatter {
	pub fn from(blob: &[u8]) -> Result<Self, EnumMatterError> {
		let header = EnumMatterHeader::from(blob)?;
		let aux_data_size = header
			.aux()
			.checked_mul(EnumMatterHeader::CELL_SIZE)
			.ok_or(EnumMatterError::Overflow)?;
		let row_data_size = header
			.cols()
			.checked_mul(header.rows())
			.ok_or(EnumMatterError::Overflow)?
			.checked_mul(EnumMatterHeader::CELL_SIZE)
			.ok_or(EnumMatterError::Overflow)?;

		let expect_len = EnumMatterHeader::HEADER_SIZE
			.checked_add(aux_data_size)
			.ok_or(EnumMatterError::Overflow)?
			.checked_add(row_data_size)
			.ok_or(EnumMatterError::Overflow)?;

		let blob_len = blob.len();
		if blob_len != expect_len {
			return Err(EnumMatterError::BadBody { expect: expect_len, got: blob_len });
		}

		let aux_offset = EnumMatterHeader::HEADER_SIZE;
		let aux_end = aux_offset + aux_data_size;
		let aux_data = blob[aux_offset..aux_end].to_vec();

		let row_offset = aux_end;
		let row_end = row_offset + row_data_size;
		let row_data = blob[row_offset..row_end].to_vec();

		Ok(Self { header, aux_data, row_data })
	}

	#[inline]
	pub fn aux(&self) -> usize {
		self.header.aux()
	}

	#[inline]
	pub fn cols(&self) -> usize {
		self.header.cols()
	}

	#[inline]
	pub fn rows(&self) -> usize {
		self.header.rows()
	}

	pub fn aux_at(&self, index: usize) -> Result<&[u8; 32], EnumMatterError> {
		let aux = self.aux();
		if index >= aux {
			return Err(EnumMatterError::OobAux { index });
		}
		let offset = index
			.checked_mul(EnumMatterHeader::CELL_SIZE)
			.ok_or(EnumMatterError::Overflow)?;
		let end = offset + EnumMatterHeader::CELL_SIZE;
		let slice: &[u8; 32] = self
			.aux_data
			.get(offset..end)
			.ok_or(EnumMatterError::OobAux { index })?
			.try_into()
			.unwrap();
		Ok(slice)
	}

	pub fn cell_at(&self, row: usize, col: usize) -> Result<&[u8; 32], EnumMatterError> {
		let rows = self.rows();
		let cols = self.cols();
		if row >= rows || col >= cols {
			return Err(EnumMatterError::OobCell { row, col });
		}

		let offset = row
			.checked_mul(cols)
			.ok_or(EnumMatterError::Overflow)?
			.checked_add(col)
			.ok_or(EnumMatterError::Overflow)?
			.checked_mul(EnumMatterHeader::CELL_SIZE)
			.ok_or(EnumMatterError::Overflow)?;
		let end = offset + EnumMatterHeader::CELL_SIZE;

		let slice: &[u8; 32] = self
			.row_data
			.get(offset..end)
			.ok_or(EnumMatterError::OobCell { row, col })?
			.try_into()
			.unwrap();
		Ok(slice)
	}

	pub fn row_at(&self, row: usize) -> Result<Vec<&[u8; 32]>, EnumMatterError> {
		let rows = self.rows();
		let cols = self.cols();
		if row >= rows {
			return Err(EnumMatterError::OobCell { row, col: 0 });
		}
		let mut offset = row
			.checked_mul(cols)
			.ok_or(EnumMatterError::Overflow)?
			.checked_mul(EnumMatterHeader::CELL_SIZE)
			.ok_or(EnumMatterError::Overflow)?;
		let mut out = Vec::with_capacity(cols);
		for col in 0..cols {
			let end = offset
				.checked_add(EnumMatterHeader::CELL_SIZE)
				.ok_or(EnumMatterError::Overflow)?;
			let cell: &[u8; 32] = self
				.row_data
				.get(offset..end)
				.ok_or(EnumMatterError::OobCell { row, col })?
				.try_into()
				.unwrap();
			offset = end;
			out.push(cell);
		}
		Ok(out)
	}
}
