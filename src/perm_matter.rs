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
	pub rows: u64,                  // product of perm column heights
	pub sum_heights: usize,         // sum of all column heights
}

impl PermHeader {
	pub const MAGIC: [u8; 4] = *b"PERM";
	pub const CELL_SIZE: usize = 32;
	pub const HEADER_SIZE_MIN: usize = 32;
	pub const HEADER_SIZE_MAX: usize = 64;

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
			for i in 0..(cols_cnt as usize) {
				let h = u16::from_le_bytes(blob[32 + i * 2..34 + i * 2].try_into().unwrap());
				col_heights.push(h);
			}
		}

		let aux = aux_types[0..(aux_cnt as usize)].to_vec();

		let mut col_offset = 0usize;
		let mut perm_idx = 0u8;
		let mut rows = 1u64;
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
			rows.checked_mul(col_height as u64).ok_or(PermMatterError::Overflow)?;
			sum_heights.checked_add(col_height).ok_or(PermMatterError::Overflow)?;
			cols.push(col);
		}
		let perm_cols: Vec<PermColumn> =
			cols.iter().filter(|c| c.perm_col).map(|c| c.clone()).collect();
		Ok(Self { aux, cols, perm_cols, rows, sum_heights })
	}

	#[inline]
	pub fn aux(&self) -> usize {
		self.aux.len()
	}

	#[inline]
	pub fn cols(&self) -> usize {
		self.cols.len()
	}

	#[inline]
	pub fn rows(&self) -> u64 {
		self.rows
	}

	#[inline]
	pub fn col_info(&self, col: usize) -> Option<&PermColumn> {
		self.cols.get(col)
	}

	#[inline]
	pub fn size(&self) -> usize {
		if self.cols.is_empty() {
			Self::HEADER_SIZE_MIN
		} else {
			Self::HEADER_SIZE_MAX
		}
	}

	#[inline]
	pub fn aux_begin(&self) -> usize {
		self.size()
	}

	#[inline]
	pub fn aux_end(&self) -> usize {
		self.size() + self.aux() * Self::CELL_SIZE
	}

	#[inline]
	pub fn col_begin(&self) -> usize {
		self.aux_begin()
	}

	#[inline]
	pub fn col_end(&self) -> usize {
		self.aux_begin() + self.sum_heights * Self::CELL_SIZE
	}

	pub fn calc_col_index(&self, row: u64) -> Result<Vec<usize>, PermMatterError> {
		let perm_idxs: Vec<usize> = self.calc_perm_col_index(row)?;
		let idxs = self
			.cols
			.iter()
			.map(|c| if c.perm_col { perm_idxs[c.perm_idx as usize] } else { row as usize })
			.collect();
		Ok(idxs)
	}

	fn calc_perm_col_index(&self, row: u64) -> Result<Vec<usize>, PermMatterError> {
		if row >= self.rows {
			return Err(PermMatterError::Overflow);
		}
		let mut idxs = Vec::with_capacity(self.perm_cols.len());
		let mut r = row;
		for (i, c) in self.perm_cols.iter().enumerate().rev() {
			let h = c.col_height as u64;
			idxs[i] = (r % h) as usize;
			r /= h;
		}
		Ok(idxs)
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

	pub fn cell_at(&self, col: usize, index: usize) -> Result<&[u8; 32], PermMatterError> {
		let Some(ci) = self.header.col_info(col) else {
			return Err(PermMatterError::OobCell { col, index });
		};
		if index >= ci.col_height {
			return Err(PermMatterError::OobCell { col, index });
		}
		let offset = (ci.col_offset + index) * PermHeader::CELL_SIZE;
		let end = offset + PermHeader::CELL_SIZE;
		let s: &[u8; 32] = self
			.col_data
			.get(offset..end)
			.ok_or(PermMatterError::OobCell { col, index })?
			.try_into()
			.unwrap();
		Ok(s)
	}

	pub fn row_at(&self, row: u64) -> Result<Vec<&[u8; 32]>, PermMatterError> {
		let idxs = self.header.calc_col_index(row)?;
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
	#[error("column {col} has zero height")]
	ZeroColumnHeight { col: usize },

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
