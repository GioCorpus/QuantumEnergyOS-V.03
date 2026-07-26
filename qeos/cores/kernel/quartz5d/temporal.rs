use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSlice {
    pub t_ns: u64,
    pub duration_ns: u64,
    pub dimension: u8,
}

#[derive(Debug, Clone, Default)]
pub struct Timeline {
    pub slices: Vec<TimeSlice>,
    pub total_duration_ns: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct TemporalAxis {
    pub origin_ns: u64,
    pub resolution_ns: u64,
}

impl TemporalAxis {
    pub const fn new(origin_ns: u64, resolution_ns: u64) -> Self {
        Self {
            origin_ns,
            resolution_ns,
        }
    }

    pub fn to_slice(&self, t_ns: u64, dimension: u8) -> TimeSlice {
        TimeSlice {
            t_ns,
            duration_ns: self.resolution_ns,
            dimension,
        }
    }
}

impl Timeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, slice: TimeSlice) {
        self.total_duration_ns += slice.duration_ns;
        self.slices.push(slice);
    }

    pub fn len(&self) -> usize {
        self.slices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slices.is_empty()
    }
}

impl fmt::Display for Timeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Timeline(slices={}, duration={}ns)",
            self.slices.len(),
            self.total_duration_ns
        )
    }
}
