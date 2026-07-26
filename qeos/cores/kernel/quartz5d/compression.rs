use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionCodec {
    None,
    Zstd { level: i32 },
    Lz4 { block_size: usize },
    Custom { ratio: f32 },
}

impl CompressionCodec {
    pub const fn ratio(&self) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Zstd { .. } => 2.5,
            Self::Lz4 { .. } => 2.0,
            Self::Custom { ratio, .. } => *ratio,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompressionRatio {
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub ratio: f32,
}

#[derive(Debug, Clone)]
pub struct QuartzCompressor {
    pub codec: CompressionCodec,
}

impl QuartzCompressor {
    pub const fn new(codec: CompressionCodec) -> Self {
        Self { codec }
    }

    pub fn compress(&self, data: &[u8]) -> CompressionRatio {
        let ratio = match self.codec {
            CompressionCodec::Lz4 { block_size } => {
                let estimate = data.len() / block_size.max(1).min(data.len()).max(1);
                data.len() as f32 / estimate.max(1) as f32
            }
            _ => self.codec.ratio(),
        };
        CompressionRatio {
            original_bytes: data.len(),
            compressed_bytes: (data.len() as f32 / ratio) as usize,
            ratio,
        }
    }
}

impl fmt::Display for CompressionCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Zstd { level } => write!(f, "zstd(level={})", level),
            Self::Lz4 { block_size } => write!(f, "lz4(block={})", block_size),
            Self::Custom { ratio } => write!(f, "custom(ratio={:.2})", ratio),
        }
    }
}
