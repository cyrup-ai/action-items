use std::io::{Read, Write};
use anyhow::Result;
use async_channel::Sender;
use bytes::Bytes;

#[derive(Clone, Debug)]
pub struct BackupConfig {
	pub compression: CompressionType,
	pub level: CompressionLevel,
	pub export_config: super::export::Config,
}

impl Default for BackupConfig {
	fn default() -> Self {
		Self {
			compression: CompressionType::default(),
			level: CompressionLevel::Default,
			export_config: super::export::Config::default(),
		}
	}
}

#[derive(Clone, Debug, Default)]
pub enum CompressionType {
	#[default]
	Gzip,
	Zstd,
	Lz4,
	None,
}

impl CompressionType {
	fn to_u8(&self) -> u8 {
		match self {
			CompressionType::None => 0,
			CompressionType::Gzip => 1,
			CompressionType::Zstd => 2,
			CompressionType::Lz4 => 3,
		}
	}

	fn from_u8(value: u8) -> Result<Self> {
		match value {
			0 => Ok(CompressionType::None),
			1 => Ok(CompressionType::Gzip),
			2 => Ok(CompressionType::Zstd),
			3 => Ok(CompressionType::Lz4),
			_ => Err(anyhow::anyhow!("Invalid compression type: {}", value)),
		}
	}
}

#[derive(Clone, Debug)]
pub enum CompressionLevel {
	Fast,
	Default,
	Best,
	Custom(u32),
}

impl CompressionLevel {
	fn to_i32(&self) -> i32 {
		match self {
			CompressionLevel::Fast => 1,
			CompressionLevel::Default => 6,
			CompressionLevel::Best => 19,
			CompressionLevel::Custom(n) => *n as i32,
		}
	}

	fn to_u8(&self) -> u8 {
		self.to_i32().min(255) as u8
	}
}

const BACKUP_MAGIC: &[u8] = b"SURREALDB_BACKUP";
const VERSION: u8 = 1;

#[derive(Debug)]
struct BackupHeader {
	magic: [u8; 16],
	version: u8,
	compression: u8,
	level: u8,
	_reserved: [u8; 13],
}

impl BackupHeader {
	fn new(compression: CompressionType, level: CompressionLevel) -> Self {
		let mut magic = [0u8; 16];
		magic.copy_from_slice(BACKUP_MAGIC);
		
		Self {
			magic,
			version: VERSION,
			compression: compression.to_u8(),
			level: level.to_u8(),
			_reserved: [0u8; 13],
		}
	}

	fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
		writer.write_all(&self.magic)?;
		writer.write_all(&[self.version, self.compression, self.level])?;
		writer.write_all(&self._reserved)?;
		Ok(())
	}

	fn read<R: Read>(reader: &mut R) -> Result<Self> {
		let mut magic = [0u8; 16];
		reader.read_exact(&mut magic)?;
		
		if &magic != BACKUP_MAGIC {
			return Err(anyhow::anyhow!("Invalid backup file: magic bytes mismatch"));
		}

		let mut version_comp_level = [0u8; 3];
		reader.read_exact(&mut version_comp_level)?;

		let version = version_comp_level[0];
		if version != VERSION {
			return Err(anyhow::anyhow!("Unsupported backup version: {}", version));
		}

		let compression = version_comp_level[1];
		let level = version_comp_level[2];

		let mut reserved = [0u8; 13];
		reader.read_exact(&mut reserved)?;

		Ok(Self {
			magic,
			version,
			compression,
			level,
			_reserved: reserved,
		})
	}

	fn compression_type(&self) -> Result<CompressionType> {
		CompressionType::from_u8(self.compression)
	}
}

pub async fn backup_with_compression(
	sender: Sender<Result<Bytes>>,
	config: BackupConfig,
) -> Result<()> {
	let mut buffer = Vec::new();

	let header = BackupHeader::new(config.compression.clone(), config.level.clone());
	header.write(&mut buffer)?;

	let level = config.level.to_i32();

	match &config.compression {
		CompressionType::Zstd => {
			use zstd::stream::write::Encoder as ZstdEncoder;
			let mut encoder = ZstdEncoder::new(&mut buffer, level)?;
			encoder.flush()?;
		}
		CompressionType::Gzip => {
			use flate2::write::GzEncoder;
			use flate2::Compression;
			let mut encoder = GzEncoder::new(&mut buffer, Compression::new(level.min(9).max(0) as u32));
			encoder.flush()?;
		}
		CompressionType::Lz4 => {
			use lz4_flex::frame::FrameEncoder;
			let mut encoder = FrameEncoder::new(&mut buffer);
			encoder.flush()?;
		}
		CompressionType::None => {}
	}

	sender.send(Ok(Bytes::from(buffer))).await.map_err(|e| anyhow::anyhow!("Failed to send backup data: {}", e))?;

	Ok(())
}

pub fn read_backup_header<R: Read>(reader: &mut R) -> Result<(CompressionType, u8)> {
	let header = BackupHeader::read(reader)?;
	let compression = header.compression_type()?;
	Ok((compression, header.level))
}
