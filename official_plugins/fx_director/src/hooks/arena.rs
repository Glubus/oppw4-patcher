use plugin_api::Oppw4PluginApi;

use crate::memory;

pub(super) const CAVE_ARENA_SIZE: usize = 0x10000;

pub(super) struct CodeCave {
    pub(super) entry: usize,
}

pub(super) struct InlineHook {
    pub(super) trampoline: usize,
}

pub(super) struct CaveArena {
    pub(super) base: usize,
    cursor: usize,
    size: usize,
}

impl CaveArena {
    pub(super) fn new(hint: usize) -> Result<Self, String> {
        let base = unsafe { memory::allocate_near_executable_block(hint, CAVE_ARENA_SIZE) }
            .ok_or_else(|| "VirtualAlloc failed for aura cave arena".to_string())?;
        Ok(Self {
            base,
            cursor: 0,
            size: CAVE_ARENA_SIZE,
        })
    }

    pub(super) fn alloc(&mut self, bytes: &[u8], alignment: usize) -> Result<usize, String> {
        let address = self.reserve(bytes.len(), alignment)?;
        unsafe { memory::write_bytes(address, bytes) };
        Ok(address)
    }

    pub(super) fn reserve(&mut self, len: usize, alignment: usize) -> Result<usize, String> {
        self.cursor = align_up(self.cursor, alignment.max(1));
        let end = self
            .cursor
            .checked_add(len)
            .ok_or_else(|| "aura cave arena offset overflow".to_string())?;
        if end > self.size {
            return Err(format!(
                "aura cave arena full requested=0x{:x} available=0x{:x}",
                len,
                self.size.saturating_sub(self.cursor)
            ));
        }
        let address = self.base + self.cursor;
        self.cursor = end;
        Ok(address)
    }

    pub(super) fn write_at(&self, address: usize, bytes: &[u8]) {
        unsafe { memory::write_bytes(address, bytes) };
    }
}

impl InlineHook {
    pub(super) unsafe fn install(
        api: &Oppw4PluginApi,
        arena: &mut CaveArena,
        site: usize,
        detour: usize,
        overwrite_len: usize,
    ) -> Result<Self, String> {
        if overwrite_len < 12 {
            return Err("inline hook overwrite_len must fit absolute jump".to_string());
        }
        let mut original = vec![0u8; overwrite_len];
        let result = api.read_memory(site, &mut original);
        if result != 0 {
            return Err(format!(
                "read_memory failed site=0x{site:x} result={result}"
            ));
        }

        let mut trampoline_code = original;
        asm::emit_abs_jmp(&mut trampoline_code, site + overwrite_len);
        let trampoline = arena.alloc(&trampoline_code, 16)?;

        let mut patch = Vec::with_capacity(overwrite_len);
        asm::emit_abs_jmp(&mut patch, detour);
        patch.resize(overwrite_len, 0x90);
        let result = api.write_memory(site, &patch);
        if result != 0 {
            return Err(format!(
                "write_memory failed site=0x{site:x} result={result}"
            ));
        }

        Ok(Self { trampoline })
    }
}

pub(super) unsafe fn patch_jump(
    api: &Oppw4PluginApi,
    site: usize,
    target: usize,
    overwrite_len: usize,
) -> Result<(), String> {
    let mut patch = vec![0x90; overwrite_len];
    asm::write_rel32_jump(&mut patch, site, 5, target)?;
    let result = api.write_memory(site, &patch);
    if result != 0 {
        return Err(format!(
            "write_memory failed site=0x{site:x} result={result}"
        ));
    }
    Ok(())
}

fn align_up(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}
