use goblin::Object;
use memflow::connector::fileio::FileIoMemory;
use memflow::error::*;
use memflow::mem::MemoryMap;
use memflow::plugins::Args;
use memflow::types::{size, Address};
use std::fs::File;
use std::io::Read;

#[cfg_attr(feature = "plugins", memflow::derive::connector(name = "kcore"))]
pub fn create_connector(args: &Args) -> Result<FileIoMemory<File>> {
    let mut mem = File::open(
        args.get("c")
            .or_else(|| args.get("core"))
            .or_else(|| args.get_default())
            .unwrap_or("/proc/kcore"),
    )
    .map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile))?;

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

        FileIoMemory::try_with_reader(mem, map)
    } else {
        Err(Error(ErrorOrigin::Connector, ErrorKind::InvalidExeFile))
    }
}
