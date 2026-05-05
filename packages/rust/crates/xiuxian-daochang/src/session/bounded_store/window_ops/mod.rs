//! Bounded session store window branch for reads and trimming.

mod conversion;
mod read_ops;
mod write_ops;

use conversion::turn_slots_to_messages;
