use crate::vm::value_source::ValueSource;

/// A slot in the memory manager
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slot {
    /// An empty slot
    Vacant {
        /// The version number of the slot
        version: u32,
    },

    /// A slot containing a value
    Occupied {
        /// The name hash of the value in the slot
        name_hash: u64,

        /// Whether the slot is write-locked
        write_locked: bool,

        /// The version number of the slot
        version: u32,

        /// The value in the slot
        value: ValueSource,
    },
}
impl Slot {
    /// Create a new vacant slot
    pub fn new() -> Self {
        Slot::Vacant { version: 1 }
    }

    /// Create a new occupied slot
    pub fn new_occupied(name_hash: u64, value: ValueSource, write_locked: bool) -> Self {
        Slot::Occupied {
            name_hash,
            write_locked,
            version: 1,
            value,
        }
    }

    /// Create a new occupied slot with a blank hash and version
    /// These values form the working stack of the VM
    pub fn new_blank(value: ValueSource) -> Self {
        Slot::Occupied {
            name_hash: 0,
            write_locked: false,
            version: 0,
            value,
        }
    }

    /// Get the version number of the slot
    pub fn version(&self) -> u32 {
        match self {
            Slot::Occupied { version, .. } => *version,
            Slot::Vacant { version } => *version,
        }
    }

    /// Check if the slot is write-locked
    pub fn write_locked(&self) -> bool {
        match self {
            Slot::Occupied { write_locked, .. } => *write_locked,
            Slot::Vacant { .. } => false,
        }
    }

    /// Convert the Slot into a Value, consuming the Slot
    pub fn into_value(self) -> Option<ValueSource> {
        match self {
            Slot::Occupied { value, .. } => Some(value),
            Slot::Vacant { .. } => None,
        }
    }

    /// Check the version number of the slot
    /// If the slot is vacant, the reference being checked is invalid
    /// so this will return false
    pub fn check_version(&self, version: u32) -> bool {
        match self {
            Slot::Occupied {
                version: slot_version,
                ..
            } => *slot_version == version,
            Slot::Vacant { .. } => false,
        }
    }

    /// Check if the name hash of the slot matches the given name hash
    /// If the slot is vacant, the reference being checked is invalid
    /// so this will return false
    pub fn check_name(&self, name_hash: u64) -> bool {
        match self {
            Slot::Occupied {
                name_hash: slot_hash,
                ..
            } => *slot_hash == name_hash,
            Slot::Vacant { .. } => false,
        }
    }

    /// Convert the Slot into a Value, consuming the Slot
    /// This is a delete operation, and will increment the version number
    pub fn take(&mut self) -> Option<ValueSource> {
        match self {
            Slot::Occupied {
                version,
                write_locked,
                ..
            } if !*write_locked => {
                // Create a new vacant slot with the next version number
                let mut s = Slot::Vacant {
                    version: version.checked_add(1).unwrap_or(1),
                };

                // Switch the existing slot with our new blank
                // slot, and return the value
                std::mem::swap(self, &mut s);
                s.into_value()
            }
            _ => None,
        }
    }

    /// Replace the value in the slot, if the slot is occupied
    pub fn put(&mut self, value: ValueSource) {
        match self {
            Slot::Occupied {
                value: slot_value,
                write_locked,
                ..
            } => {
                if !*write_locked {
                    *slot_value = value;
                }
            }

            _ => {}
        }
    }

    /// Get a reference to the value in the slot
    pub fn as_value(&self) -> Option<&ValueSource> {
        match self {
            Slot::Occupied { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Get a mutable reference to the value in the slot
    pub fn as_value_mut(&mut self) -> Option<&mut ValueSource> {
        match self {
            Slot::Occupied { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Lock the slot, preventing writes
    pub fn lock(&mut self) {
        match self {
            Slot::Occupied { write_locked, .. } => *write_locked = true,
            _ => {}
        }
    }
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Vacant { version } => write!(f, "Vacant (v={})", version),
            Slot::Occupied {
                name_hash,
                write_locked,
                version,
                value,
            } => {
                if *version == 0 {
                    write!(f, "{value:?}")
                } else {
                    if let ValueSource::Literal(crate::value::Value::Function(func)) = value {
                        write!(f, "{}FN = {func}", if *write_locked { "#RO#" } else { "" })
                    } else {
                        write!(
                            f,
                            "Occupied ({}{name_hash}v{version} = {value:?})",
                            if *write_locked { "#RO#" } else { "" }
                        )
                    }
                }
            }
        }
    }
}
