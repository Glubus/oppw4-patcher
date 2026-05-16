use std::sync::atomic::AtomicUsize;

use oppw4_script_mods::ScriptAction;

use crate::{changes::internal_hook, log, win};

const WEAPON_AURA_STOLEN_LEN: usize = 15;
const WEAPON_AURA_PATTERN: [u8; 8] = [0x44, 0x8b, 0x7b, 0x54, 0x41, 0x8b, 0x50, 0x04];
const WEAPON_AURA_EXPECTED: [u8; WEAPON_AURA_STOLEN_LEN] = [
    0x44, 0x8b, 0x7b, 0x54, 0x41, 0x8b, 0x50, 0x04, 0x48, 0x8b, 0x48, 0x18, 0x83, 0xe2, 0x01,
];

const WEAPON_AURA_DURATION_STOLEN_LEN: usize = 16;
const WEAPON_AURA_DURATION_PATTERN: [u8; 8] = [0xf3, 0x0f, 0x10, 0x9f, 0xe8, 0x02, 0x00, 0x00];
const WEAPON_AURA_DURATION_SECOND_INSTRUCTION: [u8; 4] = [0xf3, 0x0f, 0x10, 0x35];

static WEAPON_AURA_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static WEAPON_AURA_DURATION_ORIGINAL: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq)]
struct WeaponAuraConfig {
    aura_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WeaponAuraDurationConfig {
    aura_id: u32,
    step: f32,
    reset_at: f32,
    reset_to: f32,
}

pub fn install_from_actions(actions: &[ScriptAction]) -> usize {
    let Some(module) = nonzero_module() else {
        log::write_line("script-fx hooks skipped reason=no_main_module".to_string());
        return 0;
    };

    let aura = actions.iter().find_map(|action| match action {
        ScriptAction::WeaponAura { aura_id, .. } => Some(WeaponAuraConfig { aura_id: *aura_id }),
        _ => None,
    });
    let duration = actions.iter().find_map(|action| match action {
        ScriptAction::WeaponAuraDuration {
            aura_id,
            step,
            reset_at,
            reset_to,
        } => Some(WeaponAuraDurationConfig {
            aura_id: *aura_id,
            step: *step,
            reset_at: *reset_at,
            reset_to: *reset_to,
        }),
        _ => None,
    });

    let mut installed = 0;
    if let Some(config) = aura {
        installed += usize::from(unsafe { install_weapon_aura_hook(module, config) });
    }
    if let Some(config) = duration {
        installed += usize::from(unsafe { install_weapon_aura_duration_hook(module, config) });
    }
    if aura.is_some() || duration.is_some() {
        log::write_line(format!("script-fx hooks installed={installed}"));
    }
    installed
}

unsafe fn install_weapon_aura_hook(module: usize, config: WeaponAuraConfig) -> bool {
    let Some(target) = find_unique_pattern(module, &WEAPON_AURA_PATTERN, "weapon-aura") else {
        return false;
    };
    if !target_bytes_match(target, &WEAPON_AURA_EXPECTED) {
        log::write_line(format!(
            "script-fx weapon-aura hook skipped reason=unexpected_bytes target=game+0x{:x} bytes={}",
            target.saturating_sub(module),
            read_hex(target, WEAPON_AURA_STOLEN_LEN)
        ));
        return false;
    }
    let Some(return_address) = target.checked_add(WEAPON_AURA_STOLEN_LEN) else {
        log::write_line("script-fx weapon-aura hook skipped reason=return_overflow".to_string());
        return false;
    };
    let Some(stub) = build_weapon_aura_stub(return_address, config) else {
        log::write_line("script-fx weapon-aura hook skipped reason=stub_alloc_failed".to_string());
        return false;
    };
    let rva = target.saturating_sub(module);
    let installed = internal_hook::install_absolute_jump_hook(
        module,
        rva,
        WEAPON_AURA_STOLEN_LEN,
        stub,
        &WEAPON_AURA_ORIGINAL,
        "script-fx weapon-aura",
    );
    log::write_line(format!(
        "script-fx weapon-aura config aura_id={} mode=all_matching_routine",
        config.aura_id
    ));
    installed
}

unsafe fn install_weapon_aura_duration_hook(
    module: usize,
    config: WeaponAuraDurationConfig,
) -> bool {
    let Some(target) = find_unique_pattern(
        module,
        &WEAPON_AURA_DURATION_PATTERN,
        "weapon-aura-duration",
    ) else {
        return false;
    };
    if !target_bytes_match(target, &WEAPON_AURA_DURATION_PATTERN)
        || !target_bytes_match(
            target + WEAPON_AURA_DURATION_PATTERN.len(),
            &WEAPON_AURA_DURATION_SECOND_INSTRUCTION,
        )
    {
        log::write_line(
            format!(
                "script-fx weapon-aura-duration hook skipped reason=unexpected_bytes target=game+0x{:x} bytes={}",
                target.saturating_sub(module),
                read_hex(target, WEAPON_AURA_DURATION_STOLEN_LEN)
            ),
        );
        return false;
    }
    let Some(return_address) = target.checked_add(WEAPON_AURA_DURATION_STOLEN_LEN) else {
        log::write_line(
            "script-fx weapon-aura-duration hook skipped reason=return_overflow".to_string(),
        );
        return false;
    };
    let Some(const_address) =
        read_rip_relative_i32_target(target + 8, WEAPON_AURA_DURATION_STOLEN_LEN - 8)
    else {
        log::write_line(
            "script-fx weapon-aura-duration hook skipped reason=const_target_unavailable"
                .to_string(),
        );
        return false;
    };
    let Some(stub) = build_weapon_aura_duration_stub(return_address, const_address, config) else {
        log::write_line(
            "script-fx weapon-aura-duration hook skipped reason=stub_alloc_failed".to_string(),
        );
        return false;
    };
    let installed = internal_hook::install_absolute_jump_hook(
        module,
        target.saturating_sub(module),
        WEAPON_AURA_DURATION_STOLEN_LEN,
        stub,
        &WEAPON_AURA_DURATION_ORIGINAL,
        "script-fx weapon-aura-duration",
    );
    log::write_line(format!(
        "script-fx weapon-aura-duration config aura_id={} step={} reset_at={} reset_to={} const=game+0x{:x}",
        config.aura_id,
        config.step,
        config.reset_at,
        config.reset_to,
        const_address.saturating_sub(module)
    ));
    installed
}

unsafe fn find_unique_pattern(module: usize, pattern: &[u8], label: &str) -> Option<usize> {
    let Some(size) = module_image_size(module) else {
        log::write_line(format!(
            "script-fx {label} hook skipped reason=module_size_unavailable"
        ));
        return None;
    };
    let matches = find_pattern_matches(module, size, pattern);
    match matches.as_slice() {
        [address] => {
            log::write_line(format!(
                "script-fx {label} pattern match target=game+0x{:x}",
                address.saturating_sub(module)
            ));
            Some(*address)
        }
        [] => {
            log::write_line(format!(
                "script-fx {label} hook skipped reason=pattern_not_found"
            ));
            None
        }
        _ => {
            let preview = matches
                .iter()
                .take(8)
                .map(|address| format!("game+0x{:x}", address.saturating_sub(module)))
                .collect::<Vec<_>>()
                .join(",");
            log::write_line(format!(
                "script-fx {label} hook skipped reason=pattern_not_unique count={} matches=[{}]",
                matches.len(),
                preview
            ));
            None
        }
    }
}

unsafe fn module_image_size(module: usize) -> Option<usize> {
    if read_u16(module)? != 0x5a4d {
        return None;
    }
    let nt = module.checked_add(read_u32(module + 0x3c)? as usize)?;
    if read_u32(nt)? != 0x0000_4550 {
        return None;
    }
    let optional = nt.checked_add(0x18)?;
    let magic = read_u16(optional)?;
    if magic != 0x20b {
        return None;
    }
    Some(read_u32(optional + 0x38)? as usize)
}

unsafe fn read_u16(address: usize) -> Option<u16> {
    let bytes = std::slice::from_raw_parts(address as *const u8, 2);
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

unsafe fn read_u32(address: usize) -> Option<u32> {
    let bytes = std::slice::from_raw_parts(address as *const u8, 4);
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

unsafe fn read_i32(address: usize) -> Option<i32> {
    let bytes = std::slice::from_raw_parts(address as *const u8, 4);
    Some(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

unsafe fn find_pattern_matches(module: usize, size: usize, pattern: &[u8]) -> Vec<usize> {
    if module == 0 || pattern.is_empty() || size < pattern.len() {
        return Vec::new();
    }
    let image = std::slice::from_raw_parts(module as *const u8, size);
    image
        .windows(pattern.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == pattern).then_some(module + offset))
        .collect()
}

unsafe fn target_bytes_match(target: usize, expected: &[u8]) -> bool {
    let mut bytes = vec![0u8; expected.len()];
    std::ptr::copy_nonoverlapping(target as *const u8, bytes.as_mut_ptr(), bytes.len());
    bytes == expected
}

unsafe fn read_rip_relative_i32_target(
    instruction: usize,
    instruction_len: usize,
) -> Option<usize> {
    let displacement = read_i32(instruction + instruction_len - 4)? as isize;
    instruction
        .checked_add(instruction_len)?
        .checked_add_signed(displacement)
}

unsafe fn read_hex(address: usize, len: usize) -> String {
    let bytes = std::slice::from_raw_parts(address as *const u8, len);
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn build_weapon_aura_stub(return_address: usize, config: WeaponAuraConfig) -> Option<usize> {
    let mut code = Vec::new();
    push_mov_r15d_imm32(&mut code, config.aura_id);
    code.extend_from_slice(&[0x41, 0x8b, 0x50, 0x04]); // mov edx,[r8+04]
    code.extend_from_slice(&[0x48, 0x8b, 0x48, 0x18]); // mov rcx,[rax+18]
    code.extend_from_slice(&[0x83, 0xe2, 0x01]); // and edx,01
    push_abs_jump(&mut code, return_address);
    allocate_and_copy_code(&code)
}

fn build_weapon_aura_duration_stub(
    return_address: usize,
    const_address: usize,
    config: WeaponAuraDurationConfig,
) -> Option<usize> {
    let timer = win::allocate_executable_memory(4)?;
    unsafe {
        std::ptr::copy_nonoverlapping(
            config.reset_to.to_bits().to_le_bytes().as_ptr(),
            timer as *mut u8,
            4,
        );
    }

    let mut code = Vec::new();
    code.extend_from_slice(&[0x81, 0xfa]);
    code.extend_from_slice(&config.aura_id.to_le_bytes()); // cmp edx,aura_id
    let jne_pos = push_rel32_jump(&mut code, 0x85); // jne code

    push_mov_f32_to_rdi_disp(&mut code, 0x0e0, 1.0);
    push_mov_f32_to_rdi_disp(&mut code, 0x0e4, 0.0);
    push_mov_f32_to_rdi_disp(&mut code, 0x2dc, 0.0);
    push_mov_f32_to_rdi_disp(&mut code, 0x2e8, 2.0);
    push_mov_f32_to_rdi_disp(&mut code, 0x2d0, 0.0);
    push_mov_f32_to_rdi_disp(&mut code, 0x2d4, 0.0);
    push_mov_f32_to_rdi_disp(&mut code, 0x2d8, 0.0);
    push_mov_r11_imm64(&mut code, timer);
    code.extend_from_slice(&[0xf3, 0x41, 0x0f, 0x10, 0x13]); // movss xmm2,[r11]
    push_mov_eax_imm32(&mut code, config.step.to_bits());
    code.extend_from_slice(&[0x66, 0x0f, 0x6e, 0xc0]); // movd xmm0,eax
    code.extend_from_slice(&[0xf3, 0x0f, 0x58, 0xd0]); // addss xmm2,xmm0
    code.extend_from_slice(&[0xf3, 0x41, 0x0f, 0x11, 0x13]); // movss [r11],xmm2
    code.extend_from_slice(&[0xf3, 0x0f, 0x11, 0x97, 0xe4, 0x02, 0x00, 0x00]); // movss [rdi+2e4],xmm2
    push_mov_f32_to_rdi_disp(&mut code, 0x2ec, 1.0);
    push_mov_eax_imm32(&mut code, config.reset_at.to_bits());
    code.extend_from_slice(&[0x66, 0x0f, 0x6e, 0xc0]); // movd xmm0,eax
    code.extend_from_slice(&[0x0f, 0x2f, 0xd0]); // comiss xmm2,xmm0
    let jbe_pos = push_rel32_jump(&mut code, 0x86); // jbe code
    push_mov_eax_imm32(&mut code, config.reset_to.to_bits());
    code.extend_from_slice(&[0x41, 0x89, 0x03]); // mov [r11],eax

    let code_label = code.len();
    patch_rel32(&mut code, jne_pos, code_label);
    patch_rel32(&mut code, jbe_pos, code_label);
    code.extend_from_slice(&[0xf3, 0x0f, 0x10, 0x9f, 0xe8, 0x02, 0x00, 0x00]); // movss xmm3,[rdi+2e8]
    push_mov_r11_imm64(&mut code, const_address);
    code.extend_from_slice(&[0xf3, 0x41, 0x0f, 0x10, 0x33]); // movss xmm6,[r11]
    push_abs_jump(&mut code, return_address);

    allocate_and_copy_code(&code)
}

fn allocate_and_copy_code(code: &[u8]) -> Option<usize> {
    let address = win::allocate_executable_memory(code.len())?;
    unsafe {
        std::ptr::copy_nonoverlapping(code.as_ptr(), address as *mut u8, code.len());
    }
    let _ = win::flush_instruction_cache(address, code.len());
    Some(address)
}

fn push_mov_r15d_imm32(code: &mut Vec<u8>, value: u32) {
    code.extend_from_slice(&[0x41, 0xbf]);
    code.extend_from_slice(&value.to_le_bytes());
}

fn push_mov_eax_imm32(code: &mut Vec<u8>, value: u32) {
    code.push(0xb8);
    code.extend_from_slice(&value.to_le_bytes());
}

fn push_mov_r11_imm64(code: &mut Vec<u8>, value: usize) {
    code.extend_from_slice(&[0x49, 0xbb]);
    code.extend_from_slice(&(value as u64).to_le_bytes());
}

fn push_mov_f32_to_rdi_disp(code: &mut Vec<u8>, displacement: u32, value: f32) {
    code.extend_from_slice(&[0xc7, 0x87]);
    code.extend_from_slice(&displacement.to_le_bytes());
    code.extend_from_slice(&value.to_bits().to_le_bytes());
}

fn push_abs_jump(code: &mut Vec<u8>, destination: usize) {
    code.extend_from_slice(&[0xff, 0x25, 0x00, 0x00, 0x00, 0x00]);
    code.extend_from_slice(&(destination as u64).to_le_bytes());
}

fn push_rel32_jump(code: &mut Vec<u8>, condition: u8) -> usize {
    code.extend_from_slice(&[0x0f, condition]);
    let position = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    position
}

fn patch_rel32(code: &mut [u8], displacement_pos: usize, destination: usize) {
    let next = displacement_pos + 4;
    let displacement = destination as isize - next as isize;
    code[displacement_pos..displacement_pos + 4]
        .copy_from_slice(&(displacement as i32).to_le_bytes());
}

fn nonzero_module() -> Option<usize> {
    let module = win::main_module() as usize;
    (module != 0).then_some(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oppw4_script_mods::{AuraTarget, ScriptAction};

    #[test]
    fn weapon_aura_stub_contains_configured_aura_id() {
        let return_address = 0x14121d515usize;
        let stub = build_weapon_aura_stub_bytes_for_test(2830, return_address);

        assert!(stub
            .windows(4)
            .any(|window| window == 2830u32.to_le_bytes()));
        assert!(stub.ends_with(&return_address.to_le_bytes()));
    }

    #[test]
    fn weapon_aura_duration_stub_uses_dynamic_addresses() {
        let return_address = 0x14121d8aeusize;
        let const_address = 0x141a71f6cusize;
        let stub = build_weapon_aura_duration_stub_bytes_for_test(
            return_address,
            const_address,
            WeaponAuraDurationConfig {
                aura_id: 2830,
                step: 0.6,
                reset_at: 1.9,
                reset_to: 0.1,
            },
        );

        assert!(stub
            .windows(8)
            .any(|window| window == const_address.to_le_bytes()));
        assert!(stub.ends_with(&return_address.to_le_bytes()));
    }

    #[test]
    fn action_scan_picks_aura_and_duration() {
        let actions = vec![
            ScriptAction::WeaponAura {
                target: AuraTarget::LocalPlayer,
                aura_id: 2830,
            },
            ScriptAction::WeaponAuraDuration {
                aura_id: 2830,
                step: 0.6,
                reset_at: 1.9,
                reset_to: 0.1,
            },
        ];

        let aura = actions.iter().find_map(|action| match action {
            ScriptAction::WeaponAura { aura_id, .. } => Some(*aura_id),
            _ => None,
        });
        let duration = actions.iter().find_map(|action| match action {
            ScriptAction::WeaponAuraDuration { step, .. } => Some(*step),
            _ => None,
        });

        assert_eq!(aura, Some(2830));
        assert_eq!(duration, Some(0.6));
    }

    fn build_weapon_aura_stub_bytes_for_test(aura_id: u32, return_address: usize) -> Vec<u8> {
        let mut code = Vec::new();
        push_mov_r15d_imm32(&mut code, aura_id);
        code.extend_from_slice(&[0x41, 0x8b, 0x50, 0x04]);
        code.extend_from_slice(&[0x48, 0x8b, 0x48, 0x18]);
        code.extend_from_slice(&[0x83, 0xe2, 0x01]);
        push_abs_jump(&mut code, return_address);
        code
    }

    fn build_weapon_aura_duration_stub_bytes_for_test(
        return_address: usize,
        const_address: usize,
        config: WeaponAuraDurationConfig,
    ) -> Vec<u8> {
        let mut code = Vec::new();
        code.extend_from_slice(&[0x81, 0xfa]);
        code.extend_from_slice(&config.aura_id.to_le_bytes());
        let jne_pos = push_rel32_jump(&mut code, 0x85);
        push_mov_f32_to_rdi_disp(&mut code, 0x0e0, 1.0);
        push_mov_f32_to_rdi_disp(&mut code, 0x0e4, 0.0);
        push_mov_f32_to_rdi_disp(&mut code, 0x2dc, 0.0);
        push_mov_f32_to_rdi_disp(&mut code, 0x2e8, 2.0);
        push_mov_f32_to_rdi_disp(&mut code, 0x2d0, 0.0);
        push_mov_f32_to_rdi_disp(&mut code, 0x2d4, 0.0);
        push_mov_f32_to_rdi_disp(&mut code, 0x2d8, 0.0);
        push_mov_r11_imm64(&mut code, 0x12345678);
        code.extend_from_slice(&[0xf3, 0x41, 0x0f, 0x10, 0x13]);
        push_mov_eax_imm32(&mut code, config.step.to_bits());
        code.extend_from_slice(&[0x66, 0x0f, 0x6e, 0xc0]);
        code.extend_from_slice(&[0xf3, 0x0f, 0x58, 0xd0]);
        code.extend_from_slice(&[0xf3, 0x41, 0x0f, 0x11, 0x13]);
        code.extend_from_slice(&[0xf3, 0x0f, 0x11, 0x97, 0xe4, 0x02, 0x00, 0x00]);
        push_mov_f32_to_rdi_disp(&mut code, 0x2ec, 1.0);
        push_mov_eax_imm32(&mut code, config.reset_at.to_bits());
        code.extend_from_slice(&[0x66, 0x0f, 0x6e, 0xc0]);
        code.extend_from_slice(&[0x0f, 0x2f, 0xd0]);
        let jbe_pos = push_rel32_jump(&mut code, 0x86);
        push_mov_eax_imm32(&mut code, config.reset_to.to_bits());
        code.extend_from_slice(&[0x41, 0x89, 0x03]);
        let code_label = code.len();
        patch_rel32(&mut code, jne_pos, code_label);
        patch_rel32(&mut code, jbe_pos, code_label);
        code.extend_from_slice(&[0xf3, 0x0f, 0x10, 0x9f, 0xe8, 0x02, 0x00, 0x00]);
        push_mov_r11_imm64(&mut code, const_address);
        code.extend_from_slice(&[0xf3, 0x41, 0x0f, 0x10, 0x33]);
        push_abs_jump(&mut code, return_address);
        code
    }
}
