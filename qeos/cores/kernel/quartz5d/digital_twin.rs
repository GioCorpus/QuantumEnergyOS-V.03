use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwinState {
    pub timestamp_ns: u64,
    pub fidelity: f32,
    pub delta: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum TwinSynchronization {
    Synchronized,
    Drifting { delta: f32 },
    Recalibrating,
}

#[derive(Debug, Clone)]
pub struct DigitalTwin {
    pub id: u64,
    pub physical_timestamp: TwinState,
    pub digital_timestamp: TwinState,
    pub sync: TwinSynchronization,
    pub history: Vec<TwinState>,
}

impl DigitalTwin {
    pub const fn new(id: u64) -> Self {
        Self {
            id,
            physical_timestamp: TwinState {
                timestamp_ns: 0,
                fidelity: 1.0,
                delta: 0.0,
            },
            digital_timestamp: TwinState {
                timestamp_ns: 0,
                fidelity: 1.0,
                delta: 0.0,
            },
            sync: TwinSynchronization::Synchronized,
            history: Vec::new(),
        }
    }

    pub fn update_physical(&mut self, state: TwinState) {
        self.physical_timestamp = state;
        self.sync = TwinSynchronization::Synchronized;
        self.history.push(state);
    }

    pub fn update_digital(&mut self, state: TwinState) {
        self.digital_timestamp = state;
    }

    pub fn fidelity(&self) -> f32 {
        self.physical_timestamp
            .fidelity
            .min(self.digital_timestamp.fidelity)
    }
}

impl fmt::Display for DigitalTwin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DigitalTwin(id={}, fidelity={:.4}, sync={:?})",
            self.id,
            self.fidelity(),
            self.sync
        )
    }
}
