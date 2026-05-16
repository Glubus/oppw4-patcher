use std::{env, fs, path::Path, process};

mod kidsdb_dump;
#[allow(dead_code)]
mod linkdata_costume_dump;
mod linkdata_entry_triage;
mod linkdata_insert;
mod linkdata_reference_csv;
mod linkdata_scan;
mod linkdata_struct_dump;

use oppw4_rdb::{
    attach_mod_file_sizes, build_virtualization_table, inflate_linkdata_entry, parse_block_tail,
    parse_linkdata, parse_name_hash_catalog, parse_payload_tail, parse_prefixed_hex_hash,
    parse_rdb, scan_archive_names_with_catalog, scan_virtualized_names_with_catalog, ArchiveScan,
    NameHashEntry, RdbAddressSuffix, RdbBlock, RdbIndex, VirtualManager, VirtualReplacement,
};

struct CliArgs {
    command: Command,
}

enum Command {
    ExportCatalog {
        dll_path: String,
        out_path: String,
    },
    Preview {
        rdb_path: String,
    },
    SearchHash {
        rdb_path: String,
        hash: u32,
    },
    ScanArchive {
        rdb_path: String,
        folder: String,
        catalog_path: Option<String>,
    },
    ScanRoot {
        patcher_root: String,
        rdb_root: String,
        catalog_path: String,
    },
    LinkDataSearch {
        linkdata_path: String,
        needle: String,
    },
    LinkDataExtractEntry {
        linkdata_path: String,
        entry_index: usize,
        out_path: String,
    },
    LinkDataInsertLawSlot(linkdata_insert::InsertCommand),
    LawSlot5AssetPackage,
    LawSlot5Preflight {
        linkdata_path: String,
    },
    LinkDataReferenceCsv(linkdata_reference_csv::ReferenceCsvCommand),
    LinkDataStructDump(linkdata_struct_dump::StructDumpCommand),
    LinkDataEntryTriage(linkdata_entry_triage::TriageCommand),
    LinkDataProximityScan(linkdata_scan::ScanConfig),
    KidsDbDump(kidsdb_dump::DumpCommand),
}

fn main() {
    let args = parse_args_or_exit();
    run_command(args.command);
}

fn parse_args_or_exit() -> CliArgs {
    match parse_args(env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            process::exit(2);
        }
    }
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<CliArgs, String> {
    let Some(rdb_path) = args.next() else {
        return Err("usage: oppw4-rdb-tools <path-to-rdb> [hash]".to_string());
    };

    if rdb_path == "--export-catalog" {
        return parse_export_catalog_command(args).map(|command| CliArgs { command });
    }

    if rdb_path == "--scan-root" {
        return parse_scan_root_command(args).map(|command| CliArgs { command });
    }

    if rdb_path == "--linkdata-search" {
        return parse_linkdata_search_command(args).map(|command| CliArgs { command });
    }

    if rdb_path == "--linkdata-extract-entry" {
        return parse_linkdata_extract_entry_command(args).map(|command| CliArgs { command });
    }

    if rdb_path == "--linkdata-insert-law-slot" {
        return linkdata_insert::parse_command(args).map(|command| CliArgs {
            command: Command::LinkDataInsertLawSlot(command),
        });
    }

    if rdb_path == "--law-slot5-asset-package" {
        return Ok(CliArgs {
            command: Command::LawSlot5AssetPackage,
        });
    }

    if rdb_path == "--law-slot5-preflight" {
        let Some(linkdata_path) = args.next() else {
            return Err("usage: oppw4-rdb --law-slot5-preflight <linkdata-bin>".to_string());
        };
        return Ok(CliArgs {
            command: Command::LawSlot5Preflight { linkdata_path },
        });
    }

    if rdb_path == "--linkdata-reference-csv" {
        return linkdata_reference_csv::parse_command(args).map(|command| CliArgs {
            command: Command::LinkDataReferenceCsv(command),
        });
    }

    if rdb_path == "--linkdata-proximity-scan" {
        return linkdata_scan::parse_command(args).map(|command| CliArgs {
            command: Command::LinkDataProximityScan(command),
        });
    }

    if rdb_path == "--linkdata-struct-dump" {
        return linkdata_struct_dump::parse_command(args).map(|command| CliArgs {
            command: Command::LinkDataStructDump(command),
        });
    }

    if rdb_path == "--linkdata-entry-triage" {
        return linkdata_entry_triage::parse_command(args).map(|command| CliArgs {
            command: Command::LinkDataEntryTriage(command),
        });
    }

    if rdb_path == "--kidsdb-dump" {
        return kidsdb_dump::parse_command(args).map(|command| CliArgs {
            command: Command::KidsDbDump(command),
        });
    }

    let command = match args.next().as_deref() {
        Some("--scan") => parse_scan_command(rdb_path, args)?,
        Some(raw_hash) => Command::SearchHash {
            rdb_path,
            hash: parse_hash(raw_hash)?,
        },
        None => Command::Preview { rdb_path },
    };

    Ok(CliArgs { command })
}

fn parse_export_catalog_command(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let Some(dll_path) = args.next() else {
        return Err(export_catalog_usage());
    };
    let Some(out_path) = args.next() else {
        return Err(export_catalog_usage());
    };

    Ok(Command::ExportCatalog { dll_path, out_path })
}

fn export_catalog_usage() -> String {
    "usage: oppw4-rdb-tools --export-catalog <source-dll> <out-file>".to_string()
}

fn parse_scan_command(
    rdb_path: String,
    mut args: impl Iterator<Item = String>,
) -> Result<Command, String> {
    let Some(folder) = args.next() else {
        return Err(scan_usage());
    };

    let mut catalog_path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--catalog" => {
                let Some(path) = args.next() else {
                    return Err(scan_usage());
                };
                catalog_path = Some(path);
            }
            _ => return Err(format!("unknown scan option: {arg}")),
        }
    }

    Ok(Command::ScanArchive {
        rdb_path,
        folder,
        catalog_path,
    })
}

fn parse_scan_root_command(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let Some(patcher_root) = args.next() else {
        return Err(scan_root_usage());
    };

    let mut rdb_root = None;
    let mut catalog_path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rdb-root" => rdb_root = args.next(),
            "--catalog" => catalog_path = args.next(),
            _ => return Err(format!("unknown scan-root option: {arg}")),
        }
    }

    Ok(Command::ScanRoot {
        patcher_root,
        rdb_root: rdb_root.ok_or_else(scan_root_usage)?,
        catalog_path: catalog_path.ok_or_else(scan_root_usage)?,
    })
}

fn parse_linkdata_search_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Command, String> {
    let Some(linkdata_path) = args.next() else {
        return Err(linkdata_search_usage());
    };
    let Some(needle) = args.next() else {
        return Err(linkdata_search_usage());
    };

    Ok(Command::LinkDataSearch {
        linkdata_path,
        needle,
    })
}

fn linkdata_search_usage() -> String {
    "usage: oppw4-rdb-tools --linkdata-search <linkdata-bin> <needle>".to_string()
}

fn parse_linkdata_extract_entry_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Command, String> {
    let Some(linkdata_path) = args.next() else {
        return Err(linkdata_extract_entry_usage());
    };
    let Some(raw_entry_index) = args.next() else {
        return Err(linkdata_extract_entry_usage());
    };
    let Some(out_path) = args.next() else {
        return Err(linkdata_extract_entry_usage());
    };
    if args.next().is_some() {
        return Err(linkdata_extract_entry_usage());
    }

    let entry_index = raw_entry_index
        .parse::<usize>()
        .map_err(|_| format!("invalid entry index: {raw_entry_index}"))?;

    Ok(Command::LinkDataExtractEntry {
        linkdata_path,
        entry_index,
        out_path,
    })
}

fn linkdata_extract_entry_usage() -> String {
    "usage: oppw4-rdb-tools --linkdata-extract-entry <linkdata-bin> <entry-index> <out-file>"
        .to_string()
}

fn scan_usage() -> String {
    "usage: oppw4-rdb-tools <path-to-rdb> --scan <folder> [--catalog <dll>]".to_string()
}

fn scan_root_usage() -> String {
    "usage: oppw4-rdb-tools --scan-root <patcher-root> --rdb-root <rdb-root> --catalog <dll>"
        .to_string()
}

fn load_rdb_or_exit(path: &str) -> RdbIndex {
    let bytes = read_file_or_exit(path, "RDB");
    match parse_rdb(&bytes) {
        Ok(index) => index,
        Err(error) => {
            eprintln!("failed to parse {path}: {error:?}");
            process::exit(1);
        }
    }
}

fn read_file_or_exit(path: &str, label: &str) -> Vec<u8> {
    match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("failed to read {label} {path}: {error}");
            process::exit(1);
        }
    }
}

fn print_index_summary(path: &str, index: &RdbIndex) {
    println!("file: {path}");
    println!(
        "first_block_offset: 0x{:x}",
        index.header.first_block_offset
    );
    println!("declared_count: {}", index.header.declared_count);
    println!("data_prefix: {}", index.header.data_prefix);
    println!("parsed_blocks: {}", index.blocks.len());
}

fn run_command(command: Command) {
    match command {
        Command::ExportCatalog { dll_path, out_path } => export_catalog(&dll_path, &out_path),
        Command::Preview { rdb_path } => {
            let index = load_rdb_or_exit(&rdb_path);
            print_index_summary(&rdb_path, &index);
            print_blocks(index.blocks.iter().take(12), false);
        }
        Command::SearchHash { rdb_path, hash } => {
            let index = load_rdb_or_exit(&rdb_path);
            print_index_summary(&rdb_path, &index);
            search_hash(&index, hash);
        }
        Command::ScanArchive {
            rdb_path,
            folder,
            catalog_path,
        } => {
            let index = load_rdb_or_exit(&rdb_path);
            print_index_summary(&rdb_path, &index);
            scan_folder(
                &index,
                &folder,
                &load_optional_catalog(catalog_path.as_deref()),
            );
        }
        Command::ScanRoot {
            patcher_root,
            rdb_root,
            catalog_path,
        } => scan_root(&patcher_root, &rdb_root, &load_catalog(&catalog_path)),
        Command::LinkDataSearch {
            linkdata_path,
            needle,
        } => search_linkdata(&linkdata_path, &needle),
        Command::LinkDataExtractEntry {
            linkdata_path,
            entry_index,
            out_path,
        } => extract_linkdata_entry(&linkdata_path, entry_index, &out_path),
        Command::LinkDataInsertLawSlot(command) => {
            if let Err(error) = linkdata_insert::run(command) {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        Command::LawSlot5AssetPackage => print_law_slot5_asset_package(),
        Command::LawSlot5Preflight { linkdata_path } => print_law_slot5_preflight(&linkdata_path),
        Command::LinkDataReferenceCsv(command) => {
            if let Err(error) = linkdata_reference_csv::run(command) {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        Command::LinkDataStructDump(command) => {
            if let Err(error) = linkdata_struct_dump::run(command) {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        Command::LinkDataEntryTriage(command) => {
            if let Err(error) = linkdata_entry_triage::run(command) {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        Command::LinkDataProximityScan(config) => {
            if let Err(error) = linkdata_scan::run(config) {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        Command::KidsDbDump(command) => {
            if let Err(error) = kidsdb_dump::run(command) {
                eprintln!("{error}");
                process::exit(1);
            }
        }
    }
}

fn print_law_slot5_preflight(linkdata_path: &str) {
    let linkdata = read_file_or_exit(linkdata_path, "LINKDATA preflight");
    let plan = oppw4_linkdata_insert::LawSlotInsertPlan::law_slot5_layout_only();
    let report = match oppw4_linkdata_insert::preflight_law_slot5(&linkdata, &plan) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
    println!(
        "{{\"event\":\"law_slot5_preflight\",\"linkdata_path\":\"{}\",\"owner\":{},\"source_variant\":{},\"target_variant\":{},\"source_model\":{},\"target_model\":{},\"source_model_name\":{},\"target_model_name\":{},\"target_model_status\":\"{}\",\"layout_slot_patchable\":{},\"source_model_row\":{},\"target_model_row\":{},\"recommended_model_target\":{},\"asset_count\":{},\"texture_count\":{},\"requires_private_route\":{}}}",
        json_str(linkdata_path),
        report.owner,
        report.source_variant,
        report.target_variant,
        report.source_model,
        report.target_model,
        json_option_str(report.source_model_name.as_deref()),
        json_option_str(report.target_model_name.as_deref()),
        model_target_status_name(report.target_model_status),
        report.layout_slot_patchable,
        json_model_row(report.source_model_row.as_ref()),
        json_model_row(report.target_model_row.as_ref()),
        json_option_u16(report.recommended_model_target),
        report.asset_count,
        report.texture_count,
        report.requires_private_route
    );
}

fn json_option_u16(value: Option<u16>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn model_target_status_name(status: oppw4_linkdata_insert::ModelTargetStatus) -> &'static str {
    match status {
        oppw4_linkdata_insert::ModelTargetStatus::Ready => "ready",
        oppw4_linkdata_insert::ModelTargetStatus::Available => "available",
        oppw4_linkdata_insert::ModelTargetStatus::OccupiedByOther => "occupied_by_other",
    }
}

fn json_option_str(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_str(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn json_model_row(row: Option<&oppw4_data_struct::entry35::ModelRow>) -> String {
    let Some(row) = row else {
        return "null".to_string();
    };
    format!(
        "{{\"row\":{},\"offset\":\"0x{:x}\",\"owner\":{},\"relation\":{}}}",
        row.row, row.offset, row.owner, row.relation
    )
}

fn print_law_slot5_asset_package() {
    let package = oppw4_linkdata_insert::law_slot5_base_law_kids_package();
    println!(
        "{{\"event\":\"law_slot5_asset_package\",\"owner\":{},\"source_variant\":{},\"target_variant\":{},\"source_model\":{},\"target_model\":{},\"preview_mapping\":{},\"preview_resource\":{},\"requires_private_route\":{},\"assets\":[{}]}}",
        package.owner,
        package.source_variant,
        package.target_variant,
        package.source_model,
        package.target_model,
        package.preview_mapping,
        package.preview_resource,
        package.requires_private_route,
        package
            .assets
            .iter()
            .map(|asset| {
                format!(
                    "{{\"archive\":\"{}\",\"name\":\"{}\",\"target_name\":\"{}\",\"hash\":\"0x{:08x}\",\"size\":{},\"sha256\":\"{}\",\"shared_official_hash\":{}}}",
                    match asset.archive {
                        oppw4_linkdata_insert::Archive::CharacterEditor => "CharacterEditor",
                        oppw4_linkdata_insert::Archive::MaterialEditor => "MaterialEditor",
                        oppw4_linkdata_insert::Archive::RRPreview => "RRPreview",
                        oppw4_linkdata_insert::Archive::ScreenLayout => "ScreenLayout",
                    },
                    json_str(&asset.name),
                    json_str(&asset.target_name),
                    asset.hash,
                    asset.size,
                    asset.sha256,
                    asset.shared_official_hash
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    );
}

fn json_str(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn export_catalog(dll_path: &str, out_path: &str) {
    let bytes = read_file_or_exit(dll_path, "catalog source");
    let catalog = parse_name_hash_catalog(&bytes);
    let text = format_catalog_entries(&catalog);
    if let Err(error) = fs::write(out_path, text) {
        eprintln!("failed to write catalog {out_path}: {error}");
        process::exit(1);
    }
    println!("catalog_entries: {}", catalog.len());
    println!("catalog_written: {out_path}");
}

fn format_catalog_entries(catalog: &[NameHashEntry]) -> String {
    let mut output = String::new();
    for entry in catalog {
        output.push_str("0x");
        output.push_str(&format!("{:08x}", entry.hash));
        output.push(',');
        output.push_str(&entry.name);
        output.push_str("\r\n");
    }
    output
}

fn search_hash(index: &RdbIndex, hash: u32) {
    let blocks: Vec<_> = index
        .blocks
        .iter()
        .filter(|block| block.primary_hash == hash)
        .collect();

    println!("search_hash: 0x{hash:08x}");
    println!("matches: {}", blocks.len());
    print_blocks(blocks.into_iter(), true);
}

fn search_linkdata(linkdata_path: &str, needle: &str) {
    let bytes = read_file_or_exit(linkdata_path, "LINKDATA");
    let index = match parse_linkdata(&bytes) {
        Ok(index) => index,
        Err(error) => {
            eprintln!("failed to parse LINKDATA {linkdata_path}: {error:?}");
            process::exit(1);
        }
    };
    let needle_bytes = needle.as_bytes();

    println!("linkdata: {linkdata_path}");
    println!("entries: {}", index.entry_count);
    println!("needle: {needle}");

    let mut matches = 0usize;
    for entry in &index.entries {
        let inflated = match inflate_linkdata_entry(&bytes, entry) {
            Ok(inflated) if !inflated.is_empty() => inflated,
            Ok(_) => continue,
            Err(_) => continue,
        };
        let mut cursor = 0usize;
        while let Some(relative) = find_bytes_from(&inflated, needle_bytes, cursor) {
            let offset = cursor + relative;
            matches += 1;
            println!(
                "match entry={} entry_table=0x{:x} data=0x{:x} inflated_offset=0x{:x} inflated_size=0x{:x}",
                entry.index,
                entry.table_offset,
                entry.data_offset,
                offset,
                inflated.len()
            );
            print_linkdata_match_preview(&inflated, offset);
            cursor = offset + needle_bytes.len();
        }
    }
    println!("matches: {matches}");
}

fn extract_linkdata_entry(linkdata_path: &str, entry_index: usize, out_path: &str) {
    let bytes = read_file_or_exit(linkdata_path, "LINKDATA");
    let index = match parse_linkdata(&bytes) {
        Ok(index) => index,
        Err(error) => {
            eprintln!("failed to parse LINKDATA {linkdata_path}: {error:?}");
            process::exit(1);
        }
    };
    let Some(entry) = index.entries.iter().find(|entry| entry.index == entry_index) else {
        eprintln!(
            "entry {entry_index} not found in LINKDATA {linkdata_path}; entries={}",
            index.entries.len()
        );
        process::exit(1);
    };
    let inflated = match inflate_linkdata_entry(&bytes, entry) {
        Ok(inflated) => inflated,
        Err(error) => {
            eprintln!("failed to inflate entry {entry_index}: {error:?}");
            process::exit(1);
        }
    };
    if let Err(error) = fs::write(out_path, &inflated) {
        eprintln!("failed to write {out_path}: {error}");
        process::exit(1);
    }
    println!(
        "{{\"event\":\"linkdata_extract_entry\",\"linkdata_path\":\"{}\",\"entry_index\":{},\"out_path\":\"{}\",\"bytes\":{}}}",
        json_str(linkdata_path),
        entry_index,
        json_str(out_path),
        inflated.len()
    );
}

fn find_bytes_from(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start >= haystack.len() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
}

fn print_linkdata_match_preview(bytes: &[u8], offset: usize) {
    let start = offset.saturating_sub(96);
    let end = (offset + 192).min(bytes.len());
    let preview = &bytes[start..end];
    println!("  preview_range=0x{start:x}..0x{end:x}");
    println!("  text: {}", text_preview(preview));
    println!("  hex:  {}", hex_preview(preview));
}

fn print_blocks<'a>(blocks: impl IntoIterator<Item = &'a RdbBlock>, verbose_payload: bool) {
    for block in blocks {
        print_block_summary(block);
        if verbose_payload {
            print_block_payload(block);
        }
    }
}

fn print_block_summary(block: &RdbBlock) {
    println!(
        "block @ 0x{offset:08x}: len=0x{length:08x} address_len=0x{address_len:x} hash=0x{hash:08x} data_offset=0x{data_offset:08x} payload_len=0x{payload_len:x}",
        offset = block.offset,
        length = block.length,
        address_len = block.field_10,
        hash = block.primary_hash,
        data_offset = block.data_offset,
        payload_len = block.payload.len(),
    );
}

fn print_block_payload(block: &RdbBlock) {
    println!("  payload_hex: {}", hex_preview(&block.payload));
    println!("  payload_text: {}", text_preview(&block.payload));
    print_scanned_tail(block);
    print_address_tail(block);
}

fn print_scanned_tail(block: &RdbBlock) {
    if let Some(tail) = parse_payload_tail(&block.payload) {
        println!(
            "  scanned_tail: {} => part_a=0x{:x} part_b=0x{:x}{}",
            tail.raw,
            tail.part_a,
            tail.part_b,
            format_suffix(&tail.suffix)
        );
    }
}

fn print_address_tail(block: &RdbBlock) {
    if let Some(tail) = parse_block_tail(block) {
        println!(
            "  address_tail: {} => part_a=0x{:x} part_b=0x{:x}{}",
            tail.raw,
            tail.part_a,
            tail.part_b,
            format_suffix(&tail.suffix)
        );
    }
}

fn format_suffix(suffix: &Option<RdbAddressSuffix>) -> String {
    match suffix {
        Some(suffix) => format!(" suffix={}0x{:x}", suffix.marker, suffix.value),
        None => String::new(),
    }
}

fn scan_folder(index: &RdbIndex, folder: &str, catalog: &[NameHashEntry]) {
    let names = match read_file_names(folder) {
        Ok(names) => names,
        Err(error) => {
            eprintln!("failed to scan {folder}: {error}");
            process::exit(1);
        }
    };

    let scanned =
        scan_virtualized_names_with_catalog(index, names.iter().map(String::as_str), catalog);
    print_scan_summary(folder, catalog.len(), &scanned);
    print_scan_entries(&scanned);
}

fn scan_root(patcher_root: &str, rdb_root: &str, catalog: &[NameHashEntry]) {
    let archive_names = read_directory_names_or_exit(patcher_root);
    let mut total_files = 0;
    let mut total_matched = 0;
    let mut total_missing = 0;
    let mut total_unresolved = 0;
    let mut replacements = Vec::new();

    println!("scan_root: {patcher_root}");
    println!("rdb_root: {rdb_root}");
    println!("catalog_entries: {}", catalog.len());

    for archive_name in archive_names {
        let rdb_path = format!("{rdb_root}\\{archive_name}.rdb");
        let folder = format!("{patcher_root}\\{archive_name}");
        let Some((index, names)) = load_archive_inputs(&archive_name, &rdb_path, &folder) else {
            continue;
        };
        let scan = scan_archive_names_with_catalog(
            &archive_name,
            &index,
            names.iter().map(String::as_str),
            catalog,
        );
        let counts = scan.counts();

        total_files += counts.total;
        total_matched += counts.matched;
        total_missing += counts.hash_missing;
        total_unresolved += counts.unresolved_names;

        print_archive_scan_summary(&scan);
        replacements.extend(build_virtualization_table(&scan, &folder));
    }

    println!(
        "total files={total_files} matched={total_matched} missing={total_missing} unresolved={total_unresolved}"
    );
    print_virtualization_summary(replacements);
}

fn load_archive_inputs(
    archive_name: &str,
    rdb_path: &str,
    folder: &str,
) -> Option<(RdbIndex, Vec<String>)> {
    let bytes = match fs::read(rdb_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            println!("archive {archive_name}: skipped, cannot read RDB {rdb_path}: {error}");
            return None;
        }
    };
    let index = match parse_rdb(&bytes) {
        Ok(index) => index,
        Err(error) => {
            println!("archive {archive_name}: skipped, cannot parse RDB {rdb_path}: {error:?}");
            return None;
        }
    };
    let names = match read_file_names(folder) {
        Ok(names) => names,
        Err(error) => {
            println!("archive {archive_name}: skipped, cannot scan folder {folder}: {error}");
            return None;
        }
    };

    Some((index, names))
}

fn print_archive_scan_summary(scan: &ArchiveScan<'_>) {
    let counts = scan.counts();
    println!(
        "archive {}: files={} matched={} missing={} unresolved={}",
        scan.archive_name,
        counts.total,
        counts.matched,
        counts.hash_missing,
        counts.unresolved_names
    );
    for file in scan.files.iter().filter(|file| file.block.is_none()) {
        match file.hash {
            Some(hash) => println!("  missing hash=0x{hash:08x} file={}", file.file_name),
            None => println!("  unresolved file={}", file.file_name),
        }
    }
}

fn print_virtualization_summary(replacements: Vec<VirtualReplacement>) {
    let replacement_count = replacements.len();
    let enriched = match attach_mod_file_sizes(replacements) {
        Ok(replacements) => replacements,
        Err(error) => {
            println!("virtualization_table: failed to read replacement sizes: {error}");
            return;
        }
    };
    let total_mod_bytes: u64 = enriched
        .iter()
        .filter_map(|replacement| replacement.mod_size)
        .sum();
    let exact_bin_size_matches = enriched
        .iter()
        .filter(|replacement| {
            replacement
                .original_bin_size
                .zip(replacement.mod_size)
                .is_some_and(|(original, replacement)| original as u64 == replacement)
        })
        .count();
    let known_original_bin_sizes = enriched
        .iter()
        .filter(|replacement| replacement.original_bin_size.is_some())
        .count();

    println!(
        "virtualization_table: replacements={replacement_count} total_mod_bytes={total_mod_bytes}"
    );
    println!(
        "virtualization_table: known_original_bin_sizes={known_original_bin_sizes} exact_size_matches={exact_bin_size_matches}"
    );
    for replacement in enriched
        .iter()
        .filter(|replacement| {
            replacement
                .original_bin_size
                .zip(replacement.mod_size)
                .is_some_and(|(original, replacement)| original as u64 != replacement)
        })
        .take(8)
    {
        println!(
            "  size_mismatch archive={} file={} original=0x{:x} mod=0x{:x} bin_offset=0x{:x}",
            replacement.archive_name,
            replacement.file_name,
            replacement.original_bin_size.unwrap(),
            replacement.mod_size.unwrap(),
            replacement.original_bin_offset.unwrap_or(0)
        );
    }
    print_first_replacement_probe(enriched);
}

fn print_first_replacement_probe(replacements: Vec<VirtualReplacement>) {
    let Some(first) = replacements.first().cloned() else {
        return;
    };
    let mut manager = VirtualManager::new(replacements);
    let Ok(Some(handle)) = manager.open_by_hash(&first.archive_name, first.hash) else {
        println!("virtualization_probe: failed to open first replacement");
        return;
    };
    let mut buffer = [0u8; 8];
    match manager.read(handle, &mut buffer) {
        Ok(read) => println!(
            "virtualization_probe: archive={} hash=0x{:08x} first_bytes={}",
            first.archive_name,
            first.hash,
            text_preview(&buffer[..read])
        ),
        Err(error) => println!("virtualization_probe: read failed: {error}"),
    }
    manager.close(handle);
}

fn read_directory_names_or_exit(folder: &str) -> Vec<String> {
    let mut names = Vec::new();
    let entries = match fs::read_dir(Path::new(folder)) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("failed to scan root {folder}: {error}");
            process::exit(1);
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                eprintln!("failed to read root entry in {folder}: {error}");
                process::exit(1);
            }
        };
        match entry.file_type() {
            Ok(file_type) if file_type.is_dir() => {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("failed to read entry type in {folder}: {error}");
                process::exit(1);
            }
        }
    }

    names.sort();
    names
}

fn print_scan_summary(
    folder: &str,
    catalog_entries: usize,
    scanned: &[oppw4_rdb::VirtualizedFile<'_>],
) {
    let matched = scanned.iter().filter(|file| file.block.is_some()).count();
    let hash_missing = scanned
        .iter()
        .filter(|file| file.hash.is_some() && file.block.is_none())
        .count();
    let named = scanned.iter().filter(|file| file.hash.is_none()).count();

    println!("scan_folder: {folder}");
    println!("files: {}", scanned.len());
    println!("catalog_entries: {catalog_entries}");
    println!("hash_matches: {matched}");
    println!("hash_missing: {hash_missing}");
    println!("non_hash_names: {named}");
}

fn print_scan_entries(scanned: &[oppw4_rdb::VirtualizedFile<'_>]) {
    for file in scanned {
        match (file.hash, file.block) {
            (Some(hash), Some(block)) => {
                println!(
                    "match hash=0x{hash:08x} file={} block=0x{:x} data_offset=0x{:x}",
                    file.file_name, block.offset, block.data_offset
                );
            }
            (Some(hash), None) => {
                println!("missing hash=0x{hash:08x} file={}", file.file_name);
            }
            (None, None) => {
                println!("name-route file={}", file.file_name);
            }
            (None, Some(_)) => unreachable!("non-hash names cannot match by hash"),
        }
    }
}

fn load_optional_catalog(path: Option<&str>) -> Vec<NameHashEntry> {
    path.map(load_catalog).unwrap_or_default()
}

fn load_catalog(path: &str) -> Vec<NameHashEntry> {
    let bytes = read_file_or_exit(path, "catalog");
    parse_name_hash_catalog(&bytes)
}

fn read_file_names(folder: &str) -> std::io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(Path::new(folder))? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

fn parse_hash(raw: &str) -> Result<u32, String> {
    let raw = raw.trim();
    if let Some(hash) = parse_prefixed_hex_hash(raw) {
        return Ok(hash);
    }

    let hex = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .unwrap_or(raw);
    u32::from_str_radix(hex, 16).map_err(|_| format!("invalid hash: {raw}"))
}

fn hex_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(96)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn text_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&byte| match byte {
            0 => '.',
            0x20..=0x7e => byte as char,
            _ => '?',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_catalog_entries_as_hash_name_lines() {
        let text = format_catalog_entries(&[
            NameHashEntry {
                name: "800_294_face_law_dressrosa_External_00.g1t".to_string(),
                hash: 0x359b9672,
            },
            NameHashEntry {
                name: "801_294_chara_law_dressrosa_External_00.g1t".to_string(),
                hash: 0x3bff0f13,
            },
        ]);

        assert_eq!(
            text,
            "0x359b9672,800_294_face_law_dressrosa_External_00.g1t\r\n\
             0x3bff0f13,801_294_chara_law_dressrosa_External_00.g1t\r\n"
        );
    }
}
