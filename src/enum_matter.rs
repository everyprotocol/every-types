use crate::{Result, Vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumMatterError {
	BadHeader,
	BadMagic([u8; 4]),
	BadVersion(u8),
	BadAuxCount(u8),
	BadColCount(u8),
	BadAuxTypes,
	BadColTypes,
	BadBody { expect: usize, got: usize },
	OobCell { row: usize, col: usize },
	OobAux { index: usize },
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
	pub body: Vec<u8>, // len = (aux + rows*cols) * 32
}

impl EnumMatter {
	pub fn from(blob: &[u8]) -> Result<Self, EnumMatterError> {
		let header = EnumMatterHeader::from(blob)?;
		let aux = header.aux();
		let cols = header.cols();
		let rows = header.rows();

		// total 32-byte slots in body = aux section + cell section
		let cell_slots = cols.checked_mul(rows).ok_or(EnumMatterError::Overflow)?;
		let total_slots = aux.checked_add(cell_slots).ok_or(EnumMatterError::Overflow)?;
		let expect_len = total_slots
			.checked_mul(EnumMatterHeader::CELL_SIZE)
			.ok_or(EnumMatterError::Overflow)?;
		let got_len = blob
			.len()
			.checked_sub(EnumMatterHeader::HEADER_SIZE)
			.ok_or(EnumMatterError::BadHeader)?;

		if got_len != expect_len {
			return Err(EnumMatterError::BadBody { expect: expect_len, got: got_len });
		}

		let body = blob[EnumMatterHeader::HEADER_SIZE..EnumMatterHeader::HEADER_SIZE + expect_len]
			.to_vec();

		Ok(Self { header, body })
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

	/// Returns a 32-byte AUX entry at `index` (0-based).
	pub fn aux_at(&self, index: usize) -> Result<&[u8; 32], EnumMatterError> {
		let aux = self.aux();
		if index >= aux {
			return Err(EnumMatterError::OobAux { index });
		}
		let offset = index
			.checked_mul(EnumMatterHeader::CELL_SIZE)
			.ok_or(EnumMatterError::Overflow)?;
		let end = offset + EnumMatterHeader::CELL_SIZE;
		// body starts right after header; aux occupies the first `aux * 32` bytes
		let slice: &[u8; 32] = self
			.body
			.get(offset..end)
			.ok_or(EnumMatterError::OobAux { index })?
			.try_into()
			.unwrap();
		Ok(slice)
	}

	/// Returns a 32-byte cell at (row, col), 0-based.
	pub fn cell_at(&self, row: usize, col: usize) -> Result<&[u8; 32], EnumMatterError> {
		let rows = self.rows();
		let cols = self.cols();
		if row >= rows || col >= cols {
			return Err(EnumMatterError::OobCell { row, col });
		}

		let aux_bytes = self
			.aux()
			.checked_mul(EnumMatterHeader::CELL_SIZE)
			.ok_or(EnumMatterError::Overflow)?;

		let idx_in_cells = row
			.checked_mul(cols)
			.ok_or(EnumMatterError::Overflow)?
			.checked_add(col)
			.ok_or(EnumMatterError::Overflow)?;

		let cell_offset = aux_bytes
			.checked_add(
				idx_in_cells
					.checked_mul(EnumMatterHeader::CELL_SIZE)
					.ok_or(EnumMatterError::Overflow)?,
			)
			.ok_or(EnumMatterError::Overflow)?;

		let end = cell_offset + EnumMatterHeader::CELL_SIZE;

		let slice: &[u8; 32] = self
			.body
			.get(cell_offset..end)
			.ok_or(EnumMatterError::OobCell { row, col })?
			.try_into()
			.unwrap();
		Ok(slice)
	}
}
