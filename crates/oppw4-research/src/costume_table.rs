pub const STATIC_DATABASE_RVA: usize = 0x1eba738;
pub const RUNTIME_DATABASE_RVA: usize = 0x1eba750;
pub const ROW_COUNT: usize = 0xfa;
pub const LAYOUT_ROW_COUNT: usize = 0x2d9;
pub const LAW_MASTER_LAYOUT_ID: u16 = 26;
pub const LAW_DUPLICATE_VARIANT_ID: u16 = 586;
pub const LAW_DUPLICATE_VARIANT_SLOT_INDEX: usize = 4;
pub const LAW_DUPLICATE_VARIANT_ACTIVE_COUNT: u8 = 5;
pub const EMPTY_LAYOUT_VARIANT_ID: u16 = 0xffff;

const STATIC_ROOT_OFFSET: usize = 0x18;
const STATIC_LAYOUT_TABLE_OFFSET: usize = 0x08;
const STATIC_ROW_TABLE_OFFSET: usize = 0x28;
const RUNTIME_ROOT_OFFSET: usize = 0x18;
const RUNTIME_ROW_TABLE_OFFSET: usize = 0x10;
const PRIMARY_ROW_STRIDE: usize = 0xdc;
const SECONDARY_ROW_STRIDE: usize = 0xc8;
const RUNTIME_ROW_STRIDE: usize = 0x68;
const LAYOUT_ROW_STRIDE: usize = 0x44;
const LAYOUT_FAMILY_OFFSET: usize = 0x14;
const LAYOUT_CATEGORY_OFFSET: usize = 0x16;
const LAYOUT_VARIANTS_OFFSET: usize = 0x28;
const LAYOUT_ACTIVE_VARIANT_COUNT_OFFSET: usize = 0x48;
const LAYOUT_FLAGS_OFFSET: usize = 0x4a;
const LAYOUT_RAW_SIZE: usize = 0x4b;
const LAYOUT_VARIANT_COUNT: usize = 0x10;
const CATEGORY_RECORD_BASE_OFFSET: usize = 0xc43c;
const CATEGORY_RECORD_STRIDE: usize = 0x34;
const CATEGORY_PRIORITY_BASE_OFFSET: usize = 0xc438;
const CATEGORY_COUNT: usize = 0x68;
const CATEGORY_RUNTIME_SLOT_BASE: usize = 10;
const CATEGORY_RUNTIME_GATE_OFFSET: usize = 400;
const CATEGORY_RUNTIME_GATE_COUNT: usize = 6;
const PRIMARY_HEAD_OFFSET: usize = 0x4c;
const PRIMARY_HEAD_SIZE: usize = 0x40;
const ROW_FLAGS_OFFSET: usize = 0xb6;
const PRIMARY_CATEGORY_A_OFFSET: usize = 0xb8;
const PRIMARY_CATEGORY_B_OFFSET: usize = 0xbc;
const PRIMARY_CATEGORY_CONTEXT_OFFSET: usize = 0xb0;
const PRIMARY_CATEGORY_CONTEXT_SIZE: usize = 0x20;
const SECONDARY_CONTEXT_OFFSET: usize = 0xd7c0;
const SECONDARY_CONTEXT_SIZE: usize = 0x10;
const SECONDARY_SORT_OFFSET: usize = 0xd7c6;
const SECONDARY_GROUP_OFFSET: usize = 0xd7c8;
const STATIC_MARKER_BASE_OFFSET: usize = 0x19a84;
const STATIC_MARKER_STRIDE: usize = 0x12;
const RUNTIME_FLAGS_OFFSET: usize = 0x1548;
const RUNTIME_STATE_OFFSET: usize = 0x154e;
const RUNTIME_CONTEXT_SIZE: usize = 0x10;

const INTERESTING_SORT_KEYS: [u16; 7] = [26, 57, 58, 111, 131, 168, 227];
const INTERESTING_LAYOUT_IDS: [u16; 20] = [
    6, 12, 26, 36, 44, 57, 58, 77, 111, 131, 136, 141, 358, 410, 411, 511, 545, 555, 559, 586,
];
const INTERESTING_CATEGORY_IDS: [u16; 2] = [12, 26];
const INTERESTING_MODEL_IDS: [u16; 10] = [26, 227, 272, 308, 410, 411, 555, 586, 559, 77];
const MATRIX_PROBE_ROWS: [u16; 4] = [133, 136, 139, 140];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeTableDump {
    pub static_database: usize,
    pub static_root: usize,
    pub static_rows: usize,
    pub runtime_database: Option<usize>,
    pub runtime_root: Option<usize>,
    pub runtime_rows: Option<usize>,
    pub rows: Vec<CostumeRowSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeLayoutTableDump {
    pub static_database: usize,
    pub static_root: usize,
    pub static_layouts: usize,
    pub static_rows: usize,
    pub runtime_database: Option<usize>,
    pub runtime_root: Option<usize>,
    pub layouts: Vec<CostumeLayoutSnapshot>,
    pub matrix_probes: Vec<CostumeLayoutMatrixProbe>,
    pub variant_owners: Vec<CostumeVariantOwner>,
    pub free_layout_candidates: Vec<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostumeRowSnapshot {
    pub row: u16,
    pub flags: u16,
    pub group: u8,
    pub sort_key: u16,
    pub primary_category_a: u8,
    pub primary_category_b: u8,
    pub marker: u8,
    pub runtime_flags: Option<u8>,
    pub runtime_state: Option<u16>,
    pub primary_head: [u8; PRIMARY_HEAD_SIZE],
    pub primary_category_context: [u8; PRIMARY_CATEGORY_CONTEXT_SIZE],
    pub secondary_context: [u8; SECONDARY_CONTEXT_SIZE],
    pub runtime_context: Option<[u8; RUNTIME_CONTEXT_SIZE]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostumeLayoutSnapshot {
    pub layout_id: u16,
    pub family: u16,
    pub category: u16,
    pub flags_4a: u8,
    pub priority: Option<u16>,
    pub category_flags: Option<u16>,
    pub category_gates: Option<[u8; CATEGORY_RUNTIME_GATE_COUNT]>,
    pub variants: [u16; LAYOUT_VARIANT_COUNT],
    pub active_variant_count: u8,
    pub raw: [u8; LAYOUT_RAW_SIZE],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostumeVariantOwner {
    pub variant_id: u16,
    pub layout_id: u16,
    pub slot_index: u8,
    pub family: u16,
    pub category: u16,
    pub active_variant_count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostumeLayoutMatrixProbe {
    pub row: u16,
    pub layout_id: u16,
    pub byte: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostumeTableDumpError {
    NullMainModule,
    NullPointer(&'static str),
    ReadFailed {
        label: &'static str,
        address: usize,
        size: usize,
    },
}

pub trait CostumeTableMemory {
    fn read_exact(&mut self, address: usize, buffer: &mut [u8]) -> bool;
}

impl CostumeTableDump {
    pub fn enabled_rows(&self) -> usize {
        self.rows.iter().filter(|row| row.is_enabled()).count()
    }

    pub fn hidden_rows(&self) -> usize {
        self.rows.iter().filter(|row| row.is_hidden()).count()
    }

    pub fn sort26_candidate_rows(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.is_sort26_candidate())
            .count()
    }

    pub fn clone_candidate_rows(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.is_clone_candidate())
            .count()
    }

    pub fn group_counts(&self) -> [usize; 10] {
        let mut counts = [0usize; 10];
        for row in &self.rows {
            if row.group < 10 {
                counts[row.group as usize] += 1;
            }
        }
        counts
    }
}

impl CostumeLayoutTableDump {
    pub fn owners_of_variant(&self, variant_id: u16) -> Vec<CostumeVariantOwner> {
        self.variant_owners
            .iter()
            .copied()
            .filter(|owner| owner.variant_id == variant_id)
            .collect()
    }

    pub fn logged_law_category_layouts(&self) -> usize {
        self.layouts
            .iter()
            .filter(|layout| layout.category == 26)
            .count()
    }

    pub fn logged_newgate_category_layouts(&self) -> usize {
        self.layouts
            .iter()
            .filter(|layout| layout.category == 12)
            .count()
    }

    pub fn layout_variant_address(&self, layout_id: u16, index: usize) -> Option<usize> {
        if layout_id as usize >= LAYOUT_ROW_COUNT || index >= LAYOUT_VARIANT_COUNT {
            return None;
        }
        Some(
            self.static_layouts
                + layout_id as usize * LAYOUT_ROW_STRIDE
                + LAYOUT_VARIANTS_OFFSET
                + index * 2,
        )
    }

    pub fn law_duplicate_variant_address(&self) -> Option<usize> {
        self.layout_variant_address(LAW_MASTER_LAYOUT_ID, LAW_DUPLICATE_VARIANT_SLOT_INDEX)
    }

    pub fn layout_active_variant_count_address(&self, layout_id: u16) -> Option<usize> {
        if layout_id as usize >= LAYOUT_ROW_COUNT {
            return None;
        }
        Some(
            self.static_layouts
                + layout_id as usize * LAYOUT_ROW_STRIDE
                + LAYOUT_ACTIVE_VARIANT_COUNT_OFFSET,
        )
    }

    pub fn law_duplicate_variant_active_count_address(&self) -> Option<usize> {
        self.layout_active_variant_count_address(LAW_MASTER_LAYOUT_ID)
    }
}

impl CostumeRowSnapshot {
    pub fn is_enabled(self) -> bool {
        self.flags & 1 != 0
    }

    pub fn is_hidden(self) -> bool {
        self.flags & 0x8000 != 0
    }

    pub fn is_sort26_candidate(self) -> bool {
        self.sort_key == 26
    }

    pub fn has_interesting_sort_key(self) -> bool {
        INTERESTING_SORT_KEYS.contains(&self.sort_key)
    }

    pub fn is_clone_candidate(self) -> bool {
        !self.is_enabled() || self.is_hidden() || self.marker == 0
    }

    pub fn should_log_default(self) -> bool {
        self.is_sort26_candidate() || self.is_enabled() || self.runtime_flags.unwrap_or(0) != 0
    }

    pub fn should_log_detail(self) -> bool {
        self.has_interesting_sort_key() || self.group == 3
    }
}

impl CostumeLayoutSnapshot {
    pub fn has_interesting_id(self) -> bool {
        INTERESTING_LAYOUT_IDS.contains(&self.layout_id)
    }

    pub fn has_interesting_category(self) -> bool {
        INTERESTING_CATEGORY_IDS.contains(&self.category)
    }

    pub fn has_interesting_variant(self) -> bool {
        self.variants
            .iter()
            .any(|variant| INTERESTING_MODEL_IDS.contains(variant))
    }

    pub fn should_log(self) -> bool {
        self.has_interesting_id()
            || self.has_interesting_category()
            || self.has_interesting_variant()
    }
}

pub fn dump_costume_table(
    main_module_base: usize,
    memory: &mut impl CostumeTableMemory,
) -> Result<CostumeTableDump, CostumeTableDumpError> {
    if main_module_base == 0 {
        return Err(CostumeTableDumpError::NullMainModule);
    }

    let static_database = read_pointer(
        memory,
        main_module_base + STATIC_DATABASE_RVA,
        "static database global",
    )?;
    let static_root = read_pointer(
        memory,
        static_database + STATIC_ROOT_OFFSET,
        "static database root",
    )?;
    let static_rows = read_pointer(
        memory,
        static_root + STATIC_ROW_TABLE_OFFSET,
        "static row table",
    )?;

    let runtime_database = read_optional_pointer(
        memory,
        main_module_base + RUNTIME_DATABASE_RVA,
        "runtime database global",
    )?;
    let (runtime_root, runtime_rows) = if let Some(runtime_database) = runtime_database {
        let root = read_optional_pointer(
            memory,
            runtime_database + RUNTIME_ROOT_OFFSET,
            "runtime database root",
        )?;
        let rows = match root {
            Some(root) => {
                read_optional_pointer(memory, root + RUNTIME_ROW_TABLE_OFFSET, "runtime row table")?
            }
            None => None,
        };
        (root, rows)
    } else {
        (None, None)
    };

    let mut rows = Vec::with_capacity(ROW_COUNT);
    for row in 0..ROW_COUNT {
        rows.push(read_row(memory, static_rows, runtime_rows, row)?);
    }

    Ok(CostumeTableDump {
        static_database,
        static_root,
        static_rows,
        runtime_database,
        runtime_root,
        runtime_rows,
        rows,
    })
}

pub fn dump_costume_layout_table(
    main_module_base: usize,
    memory: &mut impl CostumeTableMemory,
) -> Result<CostumeLayoutTableDump, CostumeTableDumpError> {
    if main_module_base == 0 {
        return Err(CostumeTableDumpError::NullMainModule);
    }

    let static_database = read_pointer(
        memory,
        main_module_base + STATIC_DATABASE_RVA,
        "static database global",
    )?;
    let static_root = read_pointer(
        memory,
        static_database + STATIC_ROOT_OFFSET,
        "static database root",
    )?;
    let static_layouts = read_pointer(
        memory,
        static_root + STATIC_LAYOUT_TABLE_OFFSET,
        "static layout table",
    )?;
    let static_rows = read_pointer(
        memory,
        static_root + STATIC_ROW_TABLE_OFFSET,
        "static row table",
    )?;

    let runtime_database = read_optional_pointer(
        memory,
        main_module_base + RUNTIME_DATABASE_RVA,
        "runtime database global",
    )?;
    let runtime_root = runtime_database
        .map(|runtime_database| {
            read_optional_pointer(
                memory,
                runtime_database + RUNTIME_ROOT_OFFSET,
                "runtime database root",
            )
        })
        .transpose()?
        .flatten();

    let mut layouts = Vec::new();
    let mut variant_owners = Vec::new();
    let mut used_variants = vec![false; LAYOUT_ROW_COUNT];
    let mut layout_scans = Vec::with_capacity(LAYOUT_ROW_COUNT);
    for layout_id in 0..LAYOUT_ROW_COUNT {
        let scan = read_layout_usage(memory, static_layouts, layout_id)?;
        for (slot_index, variant_id) in scan.variants.iter().copied().enumerate() {
            if variant_id == EMPTY_LAYOUT_VARIANT_ID {
                continue;
            }
            if (variant_id as usize) < used_variants.len() {
                used_variants[variant_id as usize] = true;
            }
            variant_owners.push(CostumeVariantOwner {
                variant_id,
                layout_id: layout_id as u16,
                slot_index: slot_index as u8,
                family: scan.family,
                category: scan.category,
                active_variant_count: scan.active_variant_count,
            });
        }
        layout_scans.push(scan);
        if let Some(layout) = read_layout(memory, static_layouts, runtime_root, layout_id)? {
            layouts.push(layout);
        }
    }
    let free_layout_candidates = layout_scans
        .iter()
        .filter(|scan| {
            scan.is_empty_candidate()
                && !used_variants
                    .get(scan.layout_id as usize)
                    .copied()
                    .unwrap_or(true)
        })
        .map(|scan| scan.layout_id)
        .collect::<Vec<_>>();

    let mut matrix_probes = Vec::new();
    for row in MATRIX_PROBE_ROWS {
        for layout_id in INTERESTING_LAYOUT_IDS {
            matrix_probes.push(read_matrix_probe(memory, static_rows, row, layout_id)?);
        }
    }

    Ok(CostumeLayoutTableDump {
        static_database,
        static_root,
        static_layouts,
        static_rows,
        runtime_database,
        runtime_root,
        layouts,
        matrix_probes,
        variant_owners,
        free_layout_candidates,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LayoutUsageScan {
    layout_id: u16,
    family: u16,
    category: u16,
    variants: [u16; LAYOUT_VARIANT_COUNT],
    active_variant_count: u8,
}

impl LayoutUsageScan {
    fn is_empty_candidate(self) -> bool {
        self.family == EMPTY_LAYOUT_VARIANT_ID
            && self.category == EMPTY_LAYOUT_VARIANT_ID
            && self.active_variant_count == 0
            && self
                .variants
                .iter()
                .all(|variant| *variant == EMPTY_LAYOUT_VARIANT_ID)
    }
}

fn read_layout_usage(
    memory: &mut impl CostumeTableMemory,
    static_layouts: usize,
    layout_id: usize,
) -> Result<LayoutUsageScan, CostumeTableDumpError> {
    let layout = static_layouts + layout_id * LAYOUT_ROW_STRIDE;
    Ok(LayoutUsageScan {
        layout_id: layout_id as u16,
        family: read_u16(memory, layout + LAYOUT_FAMILY_OFFSET, "layout family")?,
        category: read_u16(memory, layout + LAYOUT_CATEGORY_OFFSET, "layout category")?,
        variants: read_variants(memory, layout + LAYOUT_VARIANTS_OFFSET)?,
        active_variant_count: read_u8(
            memory,
            layout + LAYOUT_ACTIVE_VARIANT_COUNT_OFFSET,
            "layout active variant count",
        )?,
    })
}

fn read_row(
    memory: &mut impl CostumeTableMemory,
    static_rows: usize,
    runtime_rows: Option<usize>,
    row: usize,
) -> Result<CostumeRowSnapshot, CostumeTableDumpError> {
    let primary = static_rows + row * PRIMARY_ROW_STRIDE;
    let secondary = static_rows + row * SECONDARY_ROW_STRIDE;
    let marker = static_rows + STATIC_MARKER_BASE_OFFSET + row * STATIC_MARKER_STRIDE;
    let runtime = runtime_rows.map(|base| base + row * RUNTIME_ROW_STRIDE);

    Ok(CostumeRowSnapshot {
        row: row as u16,
        flags: read_u16(memory, primary + ROW_FLAGS_OFFSET, "row flags")?,
        group: read_u8(memory, secondary + SECONDARY_GROUP_OFFSET, "row list group")?,
        sort_key: read_u16(memory, secondary + SECONDARY_SORT_OFFSET, "row sort key")?,
        primary_category_a: read_u8(
            memory,
            primary + PRIMARY_CATEGORY_A_OFFSET,
            "row primary category a",
        )?,
        primary_category_b: read_u8(
            memory,
            primary + PRIMARY_CATEGORY_B_OFFSET,
            "row primary category b",
        )?,
        marker: read_u8(memory, marker, "row marker")?,
        runtime_flags: read_optional_u8(memory, runtime.map(|base| base + RUNTIME_FLAGS_OFFSET))?,
        runtime_state: read_optional_u16(memory, runtime.map(|base| base + RUNTIME_STATE_OFFSET))?,
        primary_head: read_array(memory, primary + PRIMARY_HEAD_OFFSET, "row primary head")?,
        primary_category_context: read_array(
            memory,
            primary + PRIMARY_CATEGORY_CONTEXT_OFFSET,
            "row primary category context",
        )?,
        secondary_context: read_array(
            memory,
            secondary + SECONDARY_CONTEXT_OFFSET,
            "row secondary context",
        )?,
        runtime_context: read_optional_array(
            memory,
            runtime.map(|base| base + RUNTIME_FLAGS_OFFSET),
            "row runtime context",
        )?,
    })
}

fn read_layout(
    memory: &mut impl CostumeTableMemory,
    static_layouts: usize,
    runtime_root: Option<usize>,
    layout_id: usize,
) -> Result<Option<CostumeLayoutSnapshot>, CostumeTableDumpError> {
    let layout = static_layouts + layout_id * LAYOUT_ROW_STRIDE;
    let family = read_u16(memory, layout + LAYOUT_FAMILY_OFFSET, "layout family")?;
    let category = read_u16(memory, layout + LAYOUT_CATEGORY_OFFSET, "layout category")?;
    let flags_4a = read_u8(memory, layout + LAYOUT_FLAGS_OFFSET, "layout flags")?;
    let priority = if layout_id < CATEGORY_COUNT {
        Some(read_u16(
            memory,
            static_layouts + CATEGORY_PRIORITY_BASE_OFFSET + layout_id * CATEGORY_RECORD_STRIDE,
            "layout priority",
        )?)
    } else {
        None
    };
    let category_flags = if (category as usize) < CATEGORY_COUNT {
        Some(read_u16(
            memory,
            static_layouts
                + CATEGORY_RECORD_BASE_OFFSET
                + category as usize * CATEGORY_RECORD_STRIDE,
            "layout category flags",
        )?)
    } else {
        None
    };
    let variants = read_variants(memory, layout + LAYOUT_VARIANTS_OFFSET)?;
    let active_variant_count = read_u8(
        memory,
        layout + LAYOUT_ACTIVE_VARIANT_COUNT_OFFSET,
        "layout active variant count",
    )?;

    let snapshot = CostumeLayoutSnapshot {
        layout_id: layout_id as u16,
        family,
        category,
        flags_4a,
        priority,
        category_flags,
        category_gates: None,
        variants,
        active_variant_count,
        raw: [0; LAYOUT_RAW_SIZE],
    };
    if !snapshot.should_log() {
        return Ok(None);
    }

    Ok(Some(CostumeLayoutSnapshot {
        category_gates: read_category_gates(memory, runtime_root, category)?,
        raw: read_array(memory, layout, "layout raw")?,
        ..snapshot
    }))
}

fn read_variants(
    memory: &mut impl CostumeTableMemory,
    address: usize,
) -> Result<[u16; LAYOUT_VARIANT_COUNT], CostumeTableDumpError> {
    let mut variants = [0u16; LAYOUT_VARIANT_COUNT];
    for (index, variant) in variants.iter_mut().enumerate() {
        *variant = read_u16(memory, address + index * 2, "layout variant")?;
    }
    Ok(variants)
}

fn read_category_gates(
    memory: &mut impl CostumeTableMemory,
    runtime_root: Option<usize>,
    category: u16,
) -> Result<Option<[u8; CATEGORY_RUNTIME_GATE_COUNT]>, CostumeTableDumpError> {
    if category as usize >= CATEGORY_COUNT {
        return Ok(None);
    }
    let Some(runtime_root) = runtime_root else {
        return Ok(None);
    };
    let slot = read_optional_pointer(
        memory,
        runtime_root + (category as usize + CATEGORY_RUNTIME_SLOT_BASE) * 8,
        "runtime category slot",
    )?;
    read_optional_array(
        memory,
        slot.map(|slot| slot + CATEGORY_RUNTIME_GATE_OFFSET),
        "runtime category gates",
    )
}

fn read_matrix_probe(
    memory: &mut impl CostumeTableMemory,
    static_rows: usize,
    row: u16,
    layout_id: u16,
) -> Result<CostumeLayoutMatrixProbe, CostumeTableDumpError> {
    Ok(CostumeLayoutMatrixProbe {
        row,
        layout_id,
        byte: read_u8(
            memory,
            static_rows
                + row as usize * PRIMARY_ROW_STRIDE
                + PRIMARY_CATEGORY_B_OFFSET
                + 1
                + layout_id as usize,
            "row/layout matrix byte",
        )?,
    })
}

fn read_pointer(
    memory: &mut impl CostumeTableMemory,
    address: usize,
    label: &'static str,
) -> Result<usize, CostumeTableDumpError> {
    let value = read_u64(memory, address, label)? as usize;
    if value == 0 {
        return Err(CostumeTableDumpError::NullPointer(label));
    }
    Ok(value)
}

fn read_optional_pointer(
    memory: &mut impl CostumeTableMemory,
    address: usize,
    label: &'static str,
) -> Result<Option<usize>, CostumeTableDumpError> {
    let value = read_u64(memory, address, label)? as usize;
    Ok((value != 0).then_some(value))
}

fn read_optional_u8(
    memory: &mut impl CostumeTableMemory,
    address: Option<usize>,
) -> Result<Option<u8>, CostumeTableDumpError> {
    address
        .map(|address| read_u8(memory, address, "optional u8"))
        .transpose()
}

fn read_optional_u16(
    memory: &mut impl CostumeTableMemory,
    address: Option<usize>,
) -> Result<Option<u16>, CostumeTableDumpError> {
    address
        .map(|address| read_u16(memory, address, "optional u16"))
        .transpose()
}

fn read_optional_array<const N: usize>(
    memory: &mut impl CostumeTableMemory,
    address: Option<usize>,
    label: &'static str,
) -> Result<Option<[u8; N]>, CostumeTableDumpError> {
    address
        .map(|address| read_array(memory, address, label))
        .transpose()
}

fn read_array<const N: usize>(
    memory: &mut impl CostumeTableMemory,
    address: usize,
    label: &'static str,
) -> Result<[u8; N], CostumeTableDumpError> {
    let mut buffer = [0u8; N];
    read_bytes(memory, address, label, &mut buffer)?;
    Ok(buffer)
}

fn read_u8(
    memory: &mut impl CostumeTableMemory,
    address: usize,
    label: &'static str,
) -> Result<u8, CostumeTableDumpError> {
    let mut buffer = [0u8; 1];
    read_bytes(memory, address, label, &mut buffer)?;
    Ok(buffer[0])
}

fn read_u16(
    memory: &mut impl CostumeTableMemory,
    address: usize,
    label: &'static str,
) -> Result<u16, CostumeTableDumpError> {
    let mut buffer = [0u8; 2];
    read_bytes(memory, address, label, &mut buffer)?;
    Ok(u16::from_le_bytes(buffer))
}

fn read_u64(
    memory: &mut impl CostumeTableMemory,
    address: usize,
    label: &'static str,
) -> Result<u64, CostumeTableDumpError> {
    let mut buffer = [0u8; 8];
    read_bytes(memory, address, label, &mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

fn read_bytes(
    memory: &mut impl CostumeTableMemory,
    address: usize,
    label: &'static str,
    buffer: &mut [u8],
) -> Result<(), CostumeTableDumpError> {
    if memory.read_exact(address, buffer) {
        Ok(())
    } else {
        Err(CostumeTableDumpError::ReadFailed {
            label,
            address,
            size: buffer.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestMemory {
        bytes: Vec<u8>,
    }

    impl TestMemory {
        fn with_size(size: usize) -> Self {
            Self {
                bytes: vec![0; size],
            }
        }

        fn write_u8(&mut self, address: usize, value: u8) {
            self.bytes[address] = value;
        }

        fn write_u16(&mut self, address: usize, value: u16) {
            self.bytes[address..address + 2].copy_from_slice(&value.to_le_bytes());
        }

        fn write_u64(&mut self, address: usize, value: u64) {
            self.bytes[address..address + 8].copy_from_slice(&value.to_le_bytes());
        }

        fn write_bytes(&mut self, address: usize, value: &[u8]) {
            self.bytes[address..address + value.len()].copy_from_slice(value);
        }
    }

    impl CostumeTableMemory for TestMemory {
        fn read_exact(&mut self, address: usize, buffer: &mut [u8]) -> bool {
            let Some(bytes) = self.bytes.get(address..address + buffer.len()) else {
                return false;
            };
            buffer.copy_from_slice(bytes);
            true
        }
    }

    #[test]
    fn dumps_static_and_runtime_row_fields() {
        let base = 0x100;
        let static_database = 0x1000;
        let static_root = 0x1100;
        let static_rows = 0x2000;
        let runtime_database = 0x1200;
        let runtime_root = 0x1300;
        let runtime_rows = 0x7000;
        let row = 3usize;
        let primary = static_rows + row * PRIMARY_ROW_STRIDE;
        let secondary = static_rows + row * SECONDARY_ROW_STRIDE;
        let runtime = runtime_rows + row * RUNTIME_ROW_STRIDE;
        let mut memory = TestMemory::with_size(0x1ec1000);

        memory.write_u64(base + STATIC_DATABASE_RVA, static_database as u64);
        memory.write_u64(static_database + STATIC_ROOT_OFFSET, static_root as u64);
        memory.write_u64(static_root + STATIC_ROW_TABLE_OFFSET, static_rows as u64);
        memory.write_u64(base + RUNTIME_DATABASE_RVA, runtime_database as u64);
        memory.write_u64(runtime_database + RUNTIME_ROOT_OFFSET, runtime_root as u64);
        memory.write_u64(runtime_root + RUNTIME_ROW_TABLE_OFFSET, runtime_rows as u64);
        memory.write_bytes(primary + PRIMARY_HEAD_OFFSET, &[0x11; PRIMARY_HEAD_SIZE]);
        memory.write_bytes(
            primary + PRIMARY_CATEGORY_CONTEXT_OFFSET,
            &[0x22; PRIMARY_CATEGORY_CONTEXT_SIZE],
        );
        memory.write_bytes(
            secondary + SECONDARY_CONTEXT_OFFSET,
            &[0x33; SECONDARY_CONTEXT_SIZE],
        );
        memory.write_bytes(
            runtime + RUNTIME_FLAGS_OFFSET,
            &[0x44; RUNTIME_CONTEXT_SIZE],
        );
        memory.write_u16(primary + ROW_FLAGS_OFFSET, 0x8001);
        memory.write_u8(primary + PRIMARY_CATEGORY_A_OFFSET, 26);
        memory.write_u8(primary + PRIMARY_CATEGORY_B_OFFSET, 5);
        memory.write_u16(secondary + SECONDARY_SORT_OFFSET, 42);
        memory.write_u8(secondary + SECONDARY_GROUP_OFFSET, 2);
        memory.write_u8(
            static_rows + STATIC_MARKER_BASE_OFFSET + row * STATIC_MARKER_STRIDE,
            1,
        );
        memory.write_u8(runtime + RUNTIME_FLAGS_OFFSET, 0x89);
        memory.write_u16(runtime + RUNTIME_STATE_OFFSET, 0x1234);

        let dump = dump_costume_table(base, &mut memory).expect("dump succeeds");
        let snapshot = dump.rows[row];

        assert_eq!(snapshot.flags, 0x8001);
        assert_eq!(snapshot.group, 2);
        assert_eq!(snapshot.sort_key, 42);
        assert_eq!(snapshot.primary_category_a, 26);
        assert_eq!(snapshot.primary_category_b, 5);
        assert_eq!(snapshot.marker, 1);
        assert_eq!(snapshot.runtime_flags, Some(0x89));
        assert_eq!(snapshot.runtime_state, Some(0x1234));
        assert_eq!(snapshot.primary_head[0], 0x11);
        assert_eq!(snapshot.primary_category_context[8], 26);
        assert_eq!(snapshot.primary_category_context[12], 5);
        assert_eq!(
            u16::from_le_bytes([snapshot.secondary_context[6], snapshot.secondary_context[7]]),
            42
        );
        assert_eq!(snapshot.secondary_context[8], 2);
        assert_eq!(snapshot.runtime_context.expect("runtime context")[0], 0x89);
        assert!(snapshot.is_enabled());
        assert!(snapshot.is_hidden());
        assert!(!snapshot.is_sort26_candidate());
    }

    #[test]
    fn dumps_layout_table_fields_and_row_matrix() {
        let base = 0x100;
        let static_database = 0x1000;
        let static_root = 0x1100;
        let runtime_database = 0x1200;
        let runtime_root = 0x1300;
        let static_layouts = 0x3000;
        let static_rows = 0x8000;
        let runtime_category_slot = 0x1d000;
        let layout_id = 57usize;
        let layout = static_layouts + layout_id * LAYOUT_ROW_STRIDE;
        let mut memory = TestMemory::with_size(0x1ec1000);

        memory.write_u64(base + STATIC_DATABASE_RVA, static_database as u64);
        memory.write_u64(static_database + STATIC_ROOT_OFFSET, static_root as u64);
        memory.write_u64(
            static_root + STATIC_LAYOUT_TABLE_OFFSET,
            static_layouts as u64,
        );
        memory.write_u64(static_root + STATIC_ROW_TABLE_OFFSET, static_rows as u64);
        memory.write_u64(base + RUNTIME_DATABASE_RVA, runtime_database as u64);
        memory.write_u64(runtime_database + RUNTIME_ROOT_OFFSET, runtime_root as u64);
        memory.write_u64(
            runtime_root + (26 + CATEGORY_RUNTIME_SLOT_BASE) * 8,
            runtime_category_slot as u64,
        );

        memory.write_bytes(layout, &[0x55; LAYOUT_RAW_SIZE]);
        memory.write_u16(layout + LAYOUT_FAMILY_OFFSET, 0x0180);
        memory.write_u16(layout + LAYOUT_CATEGORY_OFFSET, 26);
        memory.write_u8(layout + LAYOUT_ACTIVE_VARIANT_COUNT_OFFSET, 1);
        memory.write_u8(layout + LAYOUT_FLAGS_OFFSET, 0x02);
        memory.write_u16(
            static_layouts + CATEGORY_PRIORITY_BASE_OFFSET + layout_id * CATEGORY_RECORD_STRIDE,
            0x0123,
        );
        memory.write_u16(
            static_layouts + CATEGORY_RECORD_BASE_OFFSET + 26 * CATEGORY_RECORD_STRIDE,
            0x0021,
        );
        memory.write_bytes(
            runtime_category_slot + CATEGORY_RUNTIME_GATE_OFFSET,
            &[1, 2, 3, 4, 5, 6],
        );
        memory.write_u16(layout + LAYOUT_VARIANTS_OFFSET, 227);
        memory.write_u8(
            static_rows + 133 * PRIMARY_ROW_STRIDE + PRIMARY_CATEGORY_B_OFFSET + 1 + layout_id,
            0x77,
        );

        let dump = dump_costume_layout_table(base, &mut memory).expect("dump succeeds");
        let snapshot = dump
            .layouts
            .iter()
            .find(|layout| layout.layout_id == layout_id as u16)
            .expect("interesting layout is logged");

        assert_eq!(dump.static_layouts, static_layouts);
        assert_eq!(dump.logged_law_category_layouts(), 1);
        assert_eq!(snapshot.family, 0x0180);
        assert_eq!(snapshot.category, 26);
        assert_eq!(snapshot.flags_4a, 0x02);
        assert_eq!(snapshot.priority, Some(0x0123));
        assert_eq!(snapshot.category_flags, Some(0x0021));
        assert_eq!(snapshot.category_gates, Some([1, 2, 3, 4, 5, 6]));
        assert_eq!(snapshot.variants[0], 227);
        assert_eq!(snapshot.active_variant_count, 1);
        assert_eq!(snapshot.raw[0], 0x55);
        assert!(dump
            .matrix_probes
            .iter()
            .any(|probe| probe.row == 133 && probe.layout_id == 57 && probe.byte == 0x77));
        assert_eq!(
            dump.layout_variant_address(layout_id as u16, 0),
            Some(layout + LAYOUT_VARIANTS_OFFSET)
        );
        assert_eq!(
            dump.layout_active_variant_count_address(layout_id as u16),
            Some(layout + LAYOUT_ACTIVE_VARIANT_COUNT_OFFSET)
        );
    }

    #[test]
    fn scans_variant_owners_and_free_layout_candidates() {
        let base = 0x100;
        let static_database = 0x1000;
        let static_root = 0x1100;
        let static_layouts = 0x3000;
        let static_rows = 0x8000;
        let used_layout_id = 420usize;
        let free_layout_id = 640usize;
        let used_layout = static_layouts + used_layout_id * LAYOUT_ROW_STRIDE;
        let free_layout = static_layouts + free_layout_id * LAYOUT_ROW_STRIDE;
        let mut memory = TestMemory::with_size(0x1ec1000);

        memory.write_u64(base + STATIC_DATABASE_RVA, static_database as u64);
        memory.write_u64(static_database + STATIC_ROOT_OFFSET, static_root as u64);
        memory.write_u64(
            static_root + STATIC_LAYOUT_TABLE_OFFSET,
            static_layouts as u64,
        );
        memory.write_u64(static_root + STATIC_ROW_TABLE_OFFSET, static_rows as u64);

        memory.write_u16(used_layout + LAYOUT_FAMILY_OFFSET, 99);
        memory.write_u16(used_layout + LAYOUT_CATEGORY_OFFSET, 98);
        memory.write_u8(used_layout + LAYOUT_ACTIVE_VARIANT_COUNT_OFFSET, 1);
        memory.write_u16(used_layout + LAYOUT_VARIANTS_OFFSET, 587);

        memory.write_u16(free_layout + LAYOUT_FAMILY_OFFSET, EMPTY_LAYOUT_VARIANT_ID);
        memory.write_u16(
            free_layout + LAYOUT_CATEGORY_OFFSET,
            EMPTY_LAYOUT_VARIANT_ID,
        );
        memory.write_u8(free_layout + LAYOUT_ACTIVE_VARIANT_COUNT_OFFSET, 0);
        for index in 0..LAYOUT_VARIANT_COUNT {
            memory.write_u16(
                free_layout + LAYOUT_VARIANTS_OFFSET + index * 2,
                EMPTY_LAYOUT_VARIANT_ID,
            );
        }

        let dump = dump_costume_layout_table(base, &mut memory).expect("dump succeeds");

        assert_eq!(
            dump.owners_of_variant(587),
            vec![CostumeVariantOwner {
                variant_id: 587,
                layout_id: used_layout_id as u16,
                slot_index: 0,
                family: 99,
                category: 98,
                active_variant_count: 1,
            }]
        );
        assert!(dump
            .free_layout_candidates
            .contains(&(free_layout_id as u16)));
    }
}
