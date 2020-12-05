use goblin::Object;
use memflow::connector::fileio::FileIOMemory;
use memflow::connector::ConnectorArgs;
use memflow::error::*;
use memflow::mem::MemoryMap;
use memflow::types::{size, Address};
use std::fs::File;
use std::io::Read;

#[cfg_attr(feature = "inventory", memflow::derive::connector(name = "kcore"))] 
pub fn create_connector(args: &ConnectorArgs) -> Result<FileIOMemory<File>> {
    let mut mem = File::open(
        args.get("c")
            .or_else(|| args.get("core"))
            .or_else(|| args.get_default())
            .map(|v| v.as_str())
            .unwrap_or("/proc/kcore"),
    )
    .map_err(|_| Error::Other("Failed to open memory"))?;

    let mut head = vec![0; size::mb(2)];
    mem.read(&mut head).ok();

    let mut map = MemoryMap::new();

    if let Ok(Object::Elf(elf)) = Object::parse(&head) {
        for (b, s, r) in elf
            .program_headers
            .iter()
            .filter(|h| h.p_paddr != u64::MAX)
            .filter(|h| h.p_vaddr != 0)
            .map(|h| {
                (
                    Address::from(h.p_paddr),
                    h.p_filesz as usize,
                    Address::from(h.p_offset),
                )
            })
        {
            map.push_remap(b, s, r);
        }

        FileIOMemory::try_with_reader(mem, map)
    } else {
        Err(Error::Other("Failed to parse ELF header"))
    }
}
