use super::memory_manager::MemoryManager;
use crate::traits::SerializeToBytes;
use crate::value::StdFunctionSet;

const STD_MATH: &'static [u8] = include_bytes!("../../stdlib/math.bin");
const STD_SYS: &'static [u8] = include_bytes!("../../stdlib/system.bin");

pub fn load_stdlib(mem: &mut MemoryManager) {
    load_set(mem, STD_MATH);
    load_set(mem, STD_SYS);
}

fn load_set(mem: &mut MemoryManager, set: &'static [u8]) {
    match StdFunctionSet::deserialize_from_bytes(&mut set.iter().copied()) {
        Ok(set) => set.into_mem(mem),
        Err(e) => eprintln!("Failed to load stdlib: {}", e),
    }
}
