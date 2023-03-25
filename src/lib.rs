use goblin::Object;
use memflow::connector::fileio::{CloneFile, FileIoMemory};
use memflow::mem::MemoryMap;
use memflow::prelude::v1::*;
use std::fs::File;
use std::io::Read;

#[cfg_attr(feature = "plugins", connector(name = "kcore"))]
pub fn create_connector(args: &ConnectorArgs) -> Result<FileIoMemory<CloneFile>> {
    let mut mem = File::open(
        args.target
            .as_ref()
            .map(|v| v.as_ref())
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
                    h.p_filesz as umem,
                    Address::from(h.p_offset),
                )
            })
        {
            map.push_remap(b, s, r);
        }

        FileIoMemory::with_mem_map(mem.into(), map)
    } else {
        Err(Error(ErrorOrigin::Connector, ErrorKind::InvalidExeFile))
    }
}
