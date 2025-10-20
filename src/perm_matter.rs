use crate::{Result, Vec};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct PermColumn {
	pub col_idx: u8,
	pub col_type: u8,
	pub perm_col: bool,
	pub perm_idx: u8,
	pub col_offset: usize, // #cells before this column
	pub col_height: usize, // #cells in this column
}

#[derive(Debug, Clone)]
pub struct PermHeader {
	pub aux: Vec<u8>,               // aux types
	pub cols: Vec<PermColumn>,      // all columns, natural order
	pub perm_cols: Vec<PermColumn>, // only permutation columns, order reserved
	pub rows: usize,                // product of perm column heights
	pub sum_heights: usize,         // sum of all column heights
}

impl PermHeader {
	pub const MAGIC: [u8; 4] = *b"PERM";
	pub const CELL_SIZE: usize = 32;
	pub const HEADER_SIZE_MIN: usize = 32;
	pub const HEADER_SIZE_MAX: usize = 64;

	#[inline]
	pub fn aux(&self) -> usize {
		self.aux.len()
	}

	#[inline]
	pub fn cols(&self) -> usize {
		self.cols.len()
	}

	#[inline]
	pub fn rows(&self) -> usize {
		self.rows
	}

	#[inline]
	pub fn col_info(&self, col: usize) -> Option<&PermColumn> {
		self.cols.get(col)
	}

	#[inline]
	pub fn header_end(&self) -> usize {
		if self.cols.is_empty() {
			Self::HEADER_SIZE_MIN
		} else {
			Self::HEADER_SIZE_MAX
		}
	}

	#[inline]
	pub fn aux_begin(&self) -> usize {
		self.header_end()
	}

	#[inline]
	pub fn aux_end(&self) -> usize {
		self.header_end() + self.aux() * Self::CELL_SIZE
	}

	#[inline]
	pub fn col_begin(&self) -> usize {
		self.aux_begin()
	}

	#[inline]
	pub fn col_end(&self) -> usize {
		self.aux_begin() + self.sum_heights * Self::CELL_SIZE
	}

	pub fn row_to_indexes(&self, row: usize) -> Result<Vec<usize>, PermMatterError> {
		if row >= self.rows {
			return Err(PermMatterError::Overflow);
		}
		let mut idxs = Vec::with_capacity(self.cols.len());
		let mut r = row;
		for (c, ci) in self.cols.iter().enumerate().rev() {
			if ci.perm_col {
				let h = ci.col_height;
				idxs[c] = r % h;
				r /= h;
			} else {
				idxs[c] = row;
			}
		}
		Ok(idxs)
	}

	pub fn row_to_index(&self, row: usize, col: usize) -> Result<usize, PermMatterError> {
		if row >= self.rows || col >= self.cols.len() {
			return Err(PermMatterError::Overflow);
		}

		let mut r = row;
		let mut idx = 0;
		for (c, ci) in self.cols.iter().enumerate().rev() {
			if ci.perm_col {
				let h = ci.col_height;
				idx = r % h;
				r /= h;
			} else {
				idx = row;
			}
		}
		Ok(idx)
	}

	pub fn from(blob: &[u8]) -> Result<Self, PermMatterError> {
		if blob.len() < Self::HEADER_SIZE_MIN {
			return Err(PermMatterError::BadHeader);
		}

		let magic: [u8; 4] = blob[0..4].try_into().unwrap();
		if magic != Self::MAGIC {
			return Err(PermMatterError::BadMagic(magic));
		}

		let ver_aux = blob[4];
		let ver = ver_aux >> 4;
		let aux_cnt = ver_aux & 0x0F;
		if ver != 1 {
			return Err(PermMatterError::BadVersion(ver));
		}
		if aux_cnt > 8 {
			return Err(PermMatterError::BadAuxCount(aux_cnt));
		}

		let cols_cnt = blob[5];
		if cols_cnt > 16 {
			return Err(PermMatterError::BadColCount(cols_cnt));
		}

		let enum_cols = u16::from_le_bytes(blob[6..8].try_into().unwrap());

		let aux_types: [u8; 8] = blob[8..16].try_into().unwrap();
		{
			let (active, pad) = aux_types.split_at(aux_cnt as usize);
			if active.iter().any(|&t| t == 0) || pad.iter().any(|&t| t != 0) {
				return Err(PermMatterError::BadAuxTypes);
			}
		}

		let col_types: [u8; 16] = blob[16..32].try_into().unwrap();
		{
			let (active, pad) = col_types.split_at(cols_cnt as usize);
			if active.iter().any(|&t| t == 0) || pad.iter().any(|&t| t != 0) {
				return Err(PermMatterError::BadColTypes);
			}
		}

		let mut col_heights = Vec::new();
		if cols_cnt > 0 {
			if blob.len() < Self::HEADER_SIZE_MAX {
				return Err(PermMatterError::BadHeader);
			}
			let cols_cnt = cols_cnt as usize;
			for i in 0..cols_cnt {
				let h = u16::from_le_bytes(blob[32 + i * 2..34 + i * 2].try_into().unwrap());
				col_heights.push(h);
			}
			for i in cols_cnt..16 {
				let h = u16::from_le_bytes(blob[32 + i * 2..34 + i * 2].try_into().unwrap());
				if h != 0 {
					return Err(PermMatterError::BadColumnHeight { col: i });
				}
			}
		}

		let aux = aux_types[0..(aux_cnt as usize)].to_vec();

		let mut col_offset = 0usize;
		let mut perm_idx = 0u8;
		let mut rows = 1usize;
		let mut sum_heights = 0usize;
		let mut cols = Vec::new();
		for i in 0..(cols_cnt as usize) {
			let col_height = col_heights[i] as usize;
			let perm_col = (enum_cols & (1 << (15 - i)) == 0);
			let col = PermColumn {
				col_idx: i as u8,
				col_type: col_types[i],
				perm_col,
				perm_idx,
				col_offset,
				col_height,
			};
			perm_idx += if perm_col { 1 } else { 0 };
			col_offset += col_height;
			rows.checked_mul(col_height).ok_or(PermMatterError::Overflow)?;
			sum_heights.checked_add(col_height).ok_or(PermMatterError::Overflow)?;
			cols.push(col);
		}
		let perm_cols: Vec<PermColumn> =
			cols.iter().filter(|c| c.perm_col).map(|c| c.clone()).collect();
		Ok(Self { aux, cols, perm_cols, rows, sum_heights })
	}
}

#[derive(Debug, Clone)]
pub struct PermMatter {
	pub header: PermHeader, // size = 32 or 64
	pub aux_data: Vec<u8>,  // len = aux * 32
	pub col_data: Vec<u8>,  // len = sum_heights * 32 (columns laid out back-to-back)
}

impl PermMatter {
	pub fn from(blob: &[u8]) -> Result<Self, PermMatterError> {
		let header = PermHeader::from(blob)?;
		let aux_data = blob
			.get(header.aux_begin()..header.aux_end())
			.ok_or(PermMatterError::Overflow)?
			.to_vec();
		let col_data = blob
			.get(header.col_begin()..header.col_end())
			.ok_or(PermMatterError::Overflow)?
			.to_vec();
		Ok(Self { header, aux_data, col_data })
	}

	#[inline]
	pub fn aux(&self) -> usize {
		self.header.aux.len()
	}

	#[inline]
	pub fn cols(&self) -> usize {
		self.header.cols.len()
	}

	#[inline]
	pub fn rows(&self) -> usize {
		self.header.rows
	}

	pub fn aux_at(&self, index: usize) -> Result<&[u8; 32], PermMatterError> {
		if index >= self.header.aux() {
			return Err(PermMatterError::OobAux { index });
		}
		let offset = index * PermHeader::CELL_SIZE;
		let end = offset + PermHeader::CELL_SIZE;
		let s: &[u8; 32] = self
			.aux_data
			.get(offset..end)
			.ok_or(PermMatterError::OobAux { index })?
			.try_into()
			.unwrap();
		Ok(s)
	}

	pub fn cell_at(&self, row: usize, col: usize) -> Result<&[u8; 32], PermMatterError> {
		let index = self.header.row_to_index(row, col)?;
		let offset = (self.header.cols[col].col_offset + index) * PermHeader::CELL_SIZE;
		let end = offset + PermHeader::CELL_SIZE;
		let s: &[u8; 32] = self
			.col_data
			.get(offset..end)
			.ok_or(PermMatterError::OobCell { col, index })?
			.try_into()
			.unwrap();
		Ok(s)
	}

	pub fn row_at(&self, row: usize) -> Result<Vec<&[u8; 32]>, PermMatterError> {
		let idxs = self.header.row_to_indexes(row)?;
		let mut out = Vec::with_capacity(idxs.len());
		for (col, index) in idxs.into_iter().enumerate() {
			let ci = self.header.col_info(col).unwrap();
			let offset = (ci.col_offset + index) * PermHeader::CELL_SIZE;
			let end = offset + PermHeader::CELL_SIZE;
			let cell: &[u8; 32] = self
				.col_data
				.get(offset..end)
				.ok_or(PermMatterError::OobCell { col, index })?
				.try_into()
				.unwrap();
			out.push(cell);
		}
		Ok(out)
	}
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum PermMatterError {
	// Header / shape
	#[error("invalid PermMatter header")]
	BadHeader,
	#[error("bad magic: expected 'PERM' (50 45 52 4D), got {0:02X?}")]
	BadMagic([u8; 4]),
	#[error("unsupported version {0} (expected 1)")]
	BadVersion(u8),

	// Field ranges
	#[error("aux count {0} exceeds maximum of 8")]
	BadAuxCount(u8),
	#[error("column count {0} exceeds maximum of 16")]
	BadColCount(u8),

	// Layout validity
	#[error("invalid aux types layout (first N must be >0, remaining must be 0)")]
	BadAuxTypes,
	#[error("invalid column types layout (first N must be >0, remaining must be 0)")]
	BadColTypes,
	#[error(
		"enum-cols bitmap has bits set beyond declared cols (bitmap=0b{bitmap:016b}, cols={cols})"
	)]
	BadEnumBitmap { bitmap: u16, cols: u8 },

	// Column heights block (present only if cols>0)
	#[error("column heights block too short (need {need} bytes, got {got})")]
	BadHeightsBlock { need: usize, got: usize },
	#[error("column {col} has bad height")]
	BadColumnHeight { col: usize },

	// Body sizing
	#[error("invalid body length: expected {expect} bytes, got {got} bytes")]
	BadBody { expect: usize, got: usize },

	// Access
	#[error("aux index out of bounds: {index}")]
	OobAux { index: usize },
	#[error("cell out of bounds at (col={col}, index={index})")]
	OobCell { col: usize, index: usize },

	// Arithmetic / indexing
	#[error("arithmetic overflow")]
	Overflow,
}
