use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveFront {
    pub wavelength_nm: f32,
    pub amplitude: f32,
    pub phase: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct HolographicEncoder {
    pub resolution: usize,
    pub depth_layers: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct HolographicDecoder {
    pub resolution: usize,
    pub coherence_threshold: f32,
}

impl HolographicEncoder {
    pub const fn new(resolution: usize, depth_layers: usize) -> Self {
        Self {
            resolution,
            depth_layers,
        }
    }

    pub fn encode(&self, wavefront: &WaveFront) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(self.resolution * self.depth_layers);
        for _ in 0..(self.resolution * self.depth_layers) {
            buffer.push((wavefront.amplitude * 255.0) as u8);
        }
        buffer
    }
}

impl HolographicDecoder {
    pub const fn new(resolution: usize, coherence_threshold: f32) -> Self {
        Self {
            resolution,
            coherence_threshold,
        }
    }

    pub fn decode(&self, data: &[u8]) -> Option<WaveFront> {
        if data.is_empty() {
            return None;
        }
        let amplitude = data[0] as f32 / 255.0;
        Some(WaveFront {
            wavelength_nm: 520.0,
            amplitude,
            phase: 0.0,
        })
    }
}

impl fmt::Display for WaveFront {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WaveFront(wavelength={}nm, amplitude={:.2}, phase={:.2})",
            self.wavelength_nm, self.amplitude, self.phase
        )
    }
}
