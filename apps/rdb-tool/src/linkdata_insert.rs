use std::fs;

use oppw4_linkdata_insert::{insert_law_slot, Entry3PatchScope, LawSlotInsertPlan, PatchMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InsertCommand {
    input_path: String,
    output_path: String,
    mode: PatchMode,
    entry3_scope: Entry3PatchScope,
    linked_costume_binding: bool,
    entry17_record_binding: bool,
    clone_law_base: bool,
}

pub(crate) fn parse_command(
    mut args: impl Iterator<Item = String>,
) -> Result<InsertCommand, String> {
    let Some(input_path) = args.next() else {
        return Err(usage());
    };
    let Some(output_path) = args.next() else {
        return Err(usage());
    };

    let mut mode = PatchMode::InPlace;
    let mut entry3_scope = Entry3PatchScope::LayoutOnly;
    let mut linked_costume_binding = false;
    let mut entry17_record_binding = false;
    let mut clone_law_base = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--in-place" => mode = PatchMode::InPlace,
            "--rebuild" => mode = PatchMode::RebuildCompressed,
            "--raw-expanded" => mode = PatchMode::RebuildRaw,
            "--with-entry3-variant-metadata" => {
                entry3_scope = Entry3PatchScope::LayoutAndVariantMetadata;
            }
            "--with-linked-costume-binding" => linked_costume_binding = true,
            "--with-entry17-record-binding" => entry17_record_binding = true,
            "--clone-law-base" => clone_law_base = true,
            _ => return Err(format!("unknown insert option: {arg}\n{}", usage())),
        }
    }

    Ok(InsertCommand {
        input_path,
        output_path,
        mode,
        entry3_scope,
        linked_costume_binding,
        entry17_record_binding,
        clone_law_base,
    })
}

pub(crate) fn run(command: InsertCommand) -> Result<(), String> {
    let input = fs::read(&command.input_path)
        .map_err(|error| format!("failed to read {}: {error}", command.input_path))?;
    let mut plan = if command.clone_law_base {
        LawSlotInsertPlan::law_slot5_clone_base()
    } else {
        LawSlotInsertPlan::law_slot5_layout_only()
    };
    plan.mode = command.mode;
    if !command.clone_law_base {
        plan.entry3_scope = command.entry3_scope;
        plan.linked_costume_binding = command.linked_costume_binding;
        plan.entry17_record_binding = command.entry17_record_binding;
    }

    let output =
        insert_law_slot(&input, &plan).map_err(|error| format!("insert failed: {error}"))?;
    fs::write(&command.output_path, output)
        .map_err(|error| format!("failed to write {}: {error}", command.output_path))?;

    println!(
        "{{\"event\":\"linkdata_insert_law_slot\",\"input_path\":\"{}\",\"output_path\":\"{}\",\"mode\":\"{:?}\",\"entry3_scope\":\"{:?}\",\"linked_costume_binding\":{},\"entry17_record_binding\":{},\"clone_law_base\":{},\"owner\":{},\"slot_index\":{},\"source_variant\":{},\"target_variant\":{},\"target_model\":{},\"preview_mapping\":{}}}",
        json_str(&command.input_path),
        json_str(&command.output_path),
        plan.mode,
        plan.entry3_scope,
        plan.linked_costume_binding,
        plan.entry17_record_binding,
        command.clone_law_base,
        plan.owner,
        plan.slot_index,
        plan.source_variant,
        plan.target_variant,
        plan.target_model,
        plan.preview_mapping
    );
    Ok(())
}

fn json_str(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn usage() -> String {
    "usage: oppw4-rdb --linkdata-insert-law-slot <input-linkdata> <output-linkdata> [--in-place|--rebuild|--raw-expanded] [--with-entry3-variant-metadata] [--with-linked-costume-binding] [--with-entry17-record-binding] [--clone-law-base]"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_insert_command() {
        let command = parse_command(
            ["in.bin", "out.bin", "--rebuild"]
                .into_iter()
                .map(str::to_string),
        )
        .unwrap();

        assert_eq!(
            command,
            InsertCommand {
                input_path: "in.bin".to_string(),
                output_path: "out.bin".to_string(),
                mode: PatchMode::RebuildCompressed,
                entry3_scope: Entry3PatchScope::LayoutOnly,
                linked_costume_binding: false,
                entry17_record_binding: false,
                clone_law_base: false,
            }
        );
    }

    #[test]
    fn parses_linked_costume_binding_flag() {
        let command = parse_command(
            [
                "in.bin",
                "out.bin",
                "--raw-expanded",
                "--with-entry3-variant-metadata",
                "--with-linked-costume-binding",
                "--with-entry17-record-binding",
                "--clone-law-base",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .unwrap();

        assert_eq!(command.mode, PatchMode::RebuildRaw);
        assert!(command.clone_law_base);
    }
}
