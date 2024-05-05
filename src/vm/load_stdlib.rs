use super::memory_manager::MemoryManager;
use crate::traits::SerializeToBytes;
use crate::value::StdFunctionSet;

const STDLIB: &'static [u8] = include_bytes!("../../stdlib/stdlib.lbc");

pub fn load_stdlib(mem: &mut MemoryManager) {
    match StdFunctionSet::deserialize_from_bytes(&mut STDLIB.iter().copied()) {
        Ok(set) => set.into_mem(mem),
        Err(e) => eprintln!("Failed to load stdlib: {}", e),
    }
}
