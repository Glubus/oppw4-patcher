use std::{
    collections::{HashMap, HashSet},
    ffi::{c_char, c_void, CStr},
    io::SeekFrom,
    mem::size_of,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use oppw4_rdb::{ReplacementSource, VirtualHandle, VirtualManager, VirtualReplacement};
use oppw4_research::{
    costume_table::{
        self, CostumeLayoutMatrixProbe, CostumeLayoutSnapshot, CostumeLayoutTableDump,
        CostumeRowSnapshot, CostumeTableDump, CostumeTableMemory, CostumeVariantOwner,
        EMPTY_LAYOUT_VARIANT_ID, LAW_DUPLICATE_VARIANT_ACTIVE_COUNT,
        LAW_DUPLICATE_VARIANT_SLOT_INDEX, LAW_MASTER_LAYOUT_ID,
    },
    steam::{
        self, steam_apps_bool_method_import, steam_apps_import_version, steam_apps_probe_symbol,
        SteamAppsBoolMethod, SteamAppsSymbol,
    },
};

use crate::{log, win};

type Bool = i32;
type Dword = u32;
type Handle = *mut c_void;
type Lpcwstr = *const u16;
type Lpvoid = *mut c_void;
type Lpdword = *mut Dword;
type LargeInteger = i64;

const GENERIC_READ: Dword = 0x8000_0000;
const INVALID_HANDLE_VALUE: Handle = !0usize as Handle;
const FAKE_HANDLE_MASK: usize = 0xf000_0000_0000_0000;
const FAKE_HANDLE_BITS: usize = 0x1000_0000_0000_0000;
const FAKE_HANDLE_RUNTIME_MASK: usize = 0x0f00_0000_0000_0000;
const FAKE_HANDLE_RUNTIME_SHIFT: usize = 56;
const FAKE_HANDLE_ID_MASK: usize = 0x00ff_ffff_ffff_ffff;
const FILE_TYPE_DISK: Dword = 0x0000_0001;
const FILETIME_TICKS_PER_SECOND: u64 = 10_000_000;
const WINDOWS_TO_UNIX_EPOCH_SECONDS: u64 = 11_644_473_600;
const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;
const IMAGE_ORDINAL_FLAG64: u64 = 0x8000_0000_0000_0000;
const DLC_STACK_FRAME_COUNT: usize = 16;
const MAX_DLC_STACK_LOGS: usize = 320;
const STACK_CODE_CONTEXT_BYTES: usize = 96;
const MAX_STACK_CODE_CONTEXT_LOGS: usize = 32;
const INSTALL_INTERNAL_COSTUME_TRACE_HOOKS: bool = false;
const INSTALL_CLEAN_SLOT_NAVIGATION_TRACE_HOOKS: bool = false;
const INSTALL_CLEAN_COSTUME_OBJECT_UPDATE_TRACE_HOOK: bool = false;
const MENU_VISIBLE_ROW_BUILDER_RVA: usize = 0x1560e30;
const MENU_VISIBLE_ROW_BUILDER_STOLEN_LEN: usize = 14;
const COSTUME_SELECTED_HELPER_RVA: usize = 0x155e5b0;
const COSTUME_SELECTED_HELPER_STOLEN_LEN: usize = 15;
const COSTUME_ROW_SLOT_UI_HELPER_RVA: usize = 0x155ec30;
const COSTUME_ROW_SLOT_UI_HELPER_STOLEN_LEN: usize = 19;
const COSTUME_SLOT_VARIANT_HELPER_RVA: usize = 0x1559c60;
const COSTUME_SLOT_VARIANT_HELPER_STOLEN_LEN: usize = 17;
const COSTUME_SCENE_PRESENTATION_HELPER_RVA: usize = 0x14901a0;
const COSTUME_SCENE_PRESENTATION_HELPER_STOLEN_LEN: usize = 15;
const COSTUME_SCENE_AVAILABLE_CHECK_RVA: usize = 0x1490320;
const COSTUME_SCENE_AVAILABLE_CHECK_STOLEN_LEN: usize = 16;
const COSTUME_SCENE_APPLY_HELPER_RVA: usize = 0x14906a0;
const COSTUME_SCENE_APPLY_HELPER_STOLEN_LEN: usize = 17;
const COSTUME_SCENE_POST_AVAILABLE_TRANSITION_RVA: usize = 0x1490c30;
const COSTUME_SCENE_POST_AVAILABLE_TRANSITION_STOLEN_LEN: usize = 14;
const COSTUME_SCENE_STATE_REFRESH_RVA: usize = 0x1491120;
const COSTUME_SCENE_STATE_REFRESH_STOLEN_LEN: usize = 15;
const COSTUME_SCENE_UPDATE_DISPATCHER_RVA: usize = 0x1491370;
const COSTUME_SCENE_UPDATE_DISPATCHER_STOLEN_LEN: usize = 16;
const COSTUME_SCENE_PREVIEW_REFRESH_RVA: usize = 0x14926a0;
const COSTUME_SCENE_PREVIEW_REFRESH_STOLEN_LEN: usize = 15;
const COSTUME_SCENE_LIST_BUILD_RVA: usize = 0x1493220;
const COSTUME_SCENE_LIST_BUILD_STOLEN_LEN: usize = 20;
const COSTUME_SCENE_LIST_REBUILD_RVA: usize = 0x1493820;
const COSTUME_SCENE_LIST_REBUILD_STOLEN_LEN: usize = 14;
const COSTUME_SCENE_SOURCE_APPLY_RVA: usize = 0x1493b30;
const COSTUME_SCENE_SOURCE_APPLY_STOLEN_LEN: usize = 15;
const COSTUME_OBJECT_UPDATE_RVA: usize = 0x1498580;
const COSTUME_OBJECT_UPDATE_STOLEN_LEN: usize = 16;
const COSTUME_OBJECT_APPLY_READY_RVA: usize = 0x1494c20;
const COSTUME_OBJECT_APPLY_READY_STOLEN_LEN: usize = 15;
const COSTUME_OBJECT_MODEL_READY_CHECK_RVA: usize = 0x135b7c0;
const COSTUME_OBJECT_MODEL_READY_CHECK_STOLEN_LEN: usize = 15;
const COSTUME_OBJECT_REFRESH_PREVIEW_HOOK_ENABLED: bool = false;
const COSTUME_OBJECT_REFRESH_PREVIEW_RVA: usize = 0x1497f50;
const COSTUME_OBJECT_REFRESH_PREVIEW_STOLEN_LEN: usize = 17;
const COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_HOOK_ENABLED: bool = false;
const COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA: usize = 0x14986ce;
const COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_NEXT_CALLSITE_ENABLED: bool = false;
const COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_NEXT_CALLSITE_RVA: usize = 0x1498aab;
const COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_PREV_CALLSITE_ENABLED: bool = false;
const COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_PREV_CALLSITE_RVA: usize = 0x1498bc4;
const COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_LEN: usize = 5;
const COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_BYTES: [u8; 5] =
    [0xe8, 0x7d, 0xf8, 0xff, 0xff];
const COSTUME_PREVIEW_PAGE_RESET_CALLSITE_HOOK_ENABLED: bool = false;
const COSTUME_PREVIEW_PAGE_RESET_CALLSITE_RVA: usize = 0x1489ea5;
const COSTUME_PREVIEW_PAGE_RESET_CALLSITE_TARGET_RVA: usize = 0x1488570;
const COSTUME_PREVIEW_PAGE_RESET_CALLSITE_BYTES: [u8; 5] = [0xe8, 0xc6, 0xe6, 0xff, 0xff];
const COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_HOOK_ENABLED: bool = false;
const COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_RVA: usize = 0x148a8ca;
const COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_BYTES: [u8; 5] = [0xe9, 0xa1, 0xdc, 0xff, 0xff];
const COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_HOOK_ENABLED: bool = false;
const COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_RVA: usize = 0x1488610;
const COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_STOLEN_LEN: usize = 15;
const COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_BYTES: [u8; 15] = [
    0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x74, 0x24, 0x10, 0x57, 0x48, 0x83, 0xec, 0x20,
];
const COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_GLOBAL_RVA: usize = 0x1eba7f0;
const COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_STRIDE: usize = 0x68;
const COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_SPECIAL_START: u32 = 0x351;
const COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_SPECIAL_END: u32 = 0x35d;
const COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_DIMENSION_LIMIT: u32 = 0x35e;
const COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_WIDTH_OFFSET: usize = 0x28;
const COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_HEIGHT_OFFSET: usize = 0x2c;
const LAW_EXTRA_SLOT_PREVIEW_TABLE_CLONE_292_ENABLED: bool = false;
const LAW_SLOT5_REACTIVATE_STALE_PREVIEW_911_ENABLED: bool = false;
const LAW_SLOT5_REACTIVATE_911_AFTER_CLEANUP_ENABLED: bool = false;
const LAW_SLOT5_REACTIVATE_911_BEFORE_RENDER_CANDIDATE_ENABLED: bool = false;
const LAW_SLOT5_RESTORE_CHILD8_AFTER_PREVIEW_UPDATE_ENABLED: bool = false;
const LAW_SLOT5_REFRESH_PREVIEW_AFTER_LATE_READY_ENABLED: bool = false;
const LAW_SLOT5_REFRESH_PREVIEW_AT_READY_CHECKPOINT_ENABLED: bool = false;
const LAW_SLOT5_REFRESH_PREVIEW_AFTER_911_READY_WITH_CHECKPOINT_ENABLED: bool = false;
const LAW_SLOT5_TEMP_RESTORE_CONTROLLER_FOR_LATE_REFRESH_ENABLED: bool = false;
const LAW_SLOT5_EARLY_REACTIVATE_911_AT_READY_CHECKPOINT_ENABLED: bool = false;
const LAW_SLOT5_REFRESH_WHEN_WIDGET_EXISTS_BEFORE_CONTROLLER_LOSS_ENABLED: bool = false;
const LAW_SLOT5_TEMP_RESTORE_CONTROLLER_AT_WIDGET_READY_ENABLED: bool = false;
const LAW_SLOT5_TEMP_RESTORE_CONTROLLER_LOADER_LINK_AT_WIDGET_READY_ENABLED: bool = false;
const LAW_SLOT5_REPLAY_COMPANION_PREVIEW_AFTER_READY_ENABLED: bool = false;
const LAW_SLOT5_REACTIVATE_STALE_COMPANION_1957_ENABLED: bool = false;
const LAW_SLOT5_RESTORE_COMPANION_MAPPED2D8_4713_ENABLED: bool = false;
const LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID: u32 =
    LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID as u32;
const LAW_EXTRA_SLOT_PREVIEW_TABLE_MAPPING_SOURCE_ID: u32 = 294;
const LAW_EXTRA_SLOT_PREVIEW_TABLE_ONI_FALLBACK_SOURCE_ID: u32 = 308;
const COSTUME_PREVIEW_VISIBLE_BRANCH_RVA: usize = 0x148af40;
const COSTUME_PREVIEW_VISIBLE_BRANCH_STOLEN_LEN: usize = 17;
const COSTUME_PREVIEW_VISIBLE_BRANCH_COOKIE_GLOBAL_RVA: usize = 0x1db0ae0;
const COSTUME_PREVIEW_HIDDEN_BRANCH_RVA: usize = 0x148ae20;
const COSTUME_PREVIEW_HIDDEN_BRANCH_STOLEN_LEN: usize = 16;
const COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_RVA: usize = 0x148ac40;
const COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_STOLEN_LEN: usize = 17;
const COSTUME_PREVIEW_MODEL_UPDATE_RVA: usize = 0x148b5f0;
const COSTUME_PREVIEW_MODEL_UPDATE_STOLEN_LEN: usize = 15;
const COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_RVA: usize = 0x148b800;
const COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_STOLEN_LEN: usize = 15;
const COSTUME_COMPANION_PREVIEW_UPDATE_RVA: usize = 0x1489700;
const COSTUME_COMPANION_PREVIEW_UPDATE_STOLEN_LEN: usize = 19;
const COSTUME_PREVIEW_GLOBAL_DLC_GATE_RVA: usize = 0x12ffbd0;
const COSTUME_PREVIEW_GLOBAL_DLC_GATE_STOLEN_LEN: usize = 14;
const COSTUME_PREVIEW_LAYOUT_MODE_GATE_RVA: usize = 0x12f9320;
const COSTUME_PREVIEW_LAYOUT_MODE_GATE_STOLEN_LEN: usize = 15;
const COSTUME_LAYOUT_AVAILABILITY_CHECK_RVA: usize = 0x12f92c0;
const COSTUME_LAYOUT_AVAILABILITY_CHECK_STOLEN_LEN: usize = 14;
const COSTUME_PREVIEW_TAIL_UPDATE_RVA: usize = 0x148bcc0;
const COSTUME_PREVIEW_TAIL_UPDATE_STOLEN_LEN: usize = 15;
const COSTUME_PREVIEW_RESOURCE_ATTACH_RVA: usize = 0x1613030;
const COSTUME_PREVIEW_RESOURCE_ATTACH_STOLEN_LEN: usize = 14;
const COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_HOOK_ENABLED: bool = false;
const COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_RVA: usize = 0x1582c30;
const COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_HOOK_ENABLED: bool = false;
const COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_SNAPSHOT_HOOK_ENABLED: bool = false;
const COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RVA: usize = 0x1582de6;
const COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RESUME_RVA: usize = 0x1582dfb;
const COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_STOLEN_LEN: usize = 0x15;
const COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_BYTES: [u8; 0x15] = [
    0x48, 0x89, 0x5f, 0xf0, 0x44, 0x89, 0xa7, 0x70, 0x01, 0x00, 0x00, 0xc7, 0x87, 0x7c, 0x01, 0x00,
    0x00, 0x01, 0x00, 0x00, 0x00,
];
const COSTUME_PREVIEW_RESOURCE_RESET_STATE_WRITE_RVA: usize = 0x1582dea;
const COSTUME_PREVIEW_RESOURCE_RESET_MARKER_WRITE_RVA: usize = 0x1582df1;
const COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_STOLEN_LEN: usize = 15;
const COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_BYTES: [u8; 15] = [
    0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18,
];
const COSTUME_PREVIEW_RESOURCE_RESOLVE_RVA: usize = 0x1582f50;
const COSTUME_PREVIEW_RESOURCE_RESOLVE_STOLEN_LEN: usize = 17;
const COSTUME_PREVIEW_RESOURCE_RESET_BOUNDARY_HOOK_ENABLED: bool = false;
const COSTUME_PREVIEW_RESOURCE_RESET_BOUNDARY_RVA: usize = 0x1614f40;
const COSTUME_PREVIEW_RESOURCE_RESET_BOUNDARY_STOLEN_LEN: usize = 15;
const COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_HOOK_ENABLED: bool = false;
const COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RVA: usize = 0x133c19c;
const COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RESUME_RVA: usize = 0x133c19f;
const COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_LEN: usize = 5;
const COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_BYTES: [u8; 5] =
    [0xff, 0x50, 0x18, 0x48, 0x8b];
const UI_CHILD_TOGGLE_RVA: usize = 0x1604620;
const UI_CHILD_TOGGLE_STOLEN_LEN: usize = 17;
const LAUNCH_COSTUME_STATE_CONSUMER_RVA: usize = 0x1252cc0;
const LAUNCH_COSTUME_STATE_CONSUMER_STOLEN_LEN: usize = 17;
const COSTUME_SCENE_FALLBACK_LAYOUT_RVA: usize = 0x1492970;
const COSTUME_SCENE_FALLBACK_LAYOUT_STOLEN_LEN: usize = 14;
const COSTUME_SELECTION_LOOKUP_RVA: usize = 0x210b60;
const COSTUME_SELECTION_LOOKUP_STOLEN_LEN: usize = 20;
const COSTUME_SELECTION_SETTER_RVA: usize = 0x0b16c0;
const COSTUME_SELECTION_SETTER_STOLEN_LEN: usize = 16;
const COSTUME_VARIANT_UNLOCK_CHECK_RVA: usize = 0x12f52a0;
const COSTUME_VARIANT_UNLOCK_CHECK_STOLEN_LEN: usize = 16;
const COSTUME_VALIDATOR_RVA: usize = 0x12f8020;
const COSTUME_VALIDATOR_STOLEN_LEN: usize = 15;
const COSTUME_RESOLVER_RVA: usize = 0x16132b0;
const COSTUME_RESOLVER_STOLEN_LEN: usize = 15;
const ABSOLUTE_JUMP_LEN: usize = 14;
const MAX_COSTUME_MENU_TRACE_LOGS: usize = 160;
const COSTUME_MENU_UNINTERESTING_TRACE_LOGS: usize = 40;
const MAX_COSTUME_SELECTION_TRACE_LOGS: usize = 260;
const COSTUME_SELECTION_UNINTERESTING_TRACE_LOGS: usize = 32;
const MAX_COSTUME_SCENE_TRACE_LOGS: usize = 8;
const MAX_COSTUME_SCENE_IMPORTANT_TRACE_LOGS: usize = 8;
const COSTUME_SCENE_UNINTERESTING_TRACE_LOGS: usize = 32;
const MAX_COSTUME_SCENE_LIST_TRACE_LOGS: usize = 4;
const MAX_COSTUME_SCENE_LIST_IMPORTANT_TRACE_LOGS: usize = 4;
const MAX_COSTUME_SCENE_LIST_BUILD_DETAIL_TRACE_LOGS: usize = 4;
const MAX_COSTUME_SCENE_LIST_REBUILD_DETAIL_TRACE_LOGS: usize = 4;
const MAX_COSTUME_PREVIEW_VISIBLE_DECISION_TRACE_LOGS: usize = 96;
const MAX_COSTUME_SCENE_LIST_FLAG_PATCH_LOGS: usize = 64;
const MAX_COSTUME_SCENE_LOCKED_FLAG_PATCH_LOGS: usize = 96;
const MAX_COSTUME_OBJECT_UPDATE_TRACE_LOGS: usize = 48;
const COSTUME_OBJECT_UPDATE_UNINTERESTING_TRACE_LOGS: usize = 24;
const MAX_COSTUME_OBJECT_APPLY_READY_TRACE_LOGS: usize = 32;
const MAX_COSTUME_OBJECT_MODEL_READY_TRACE_LOGS: usize = 32;
const MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TRACE_LOGS: usize = 48;
const MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_LOGS: usize = 64;
const MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_SKIPPED_LOGS: usize = 4;
const MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_GUARD_LOGS: usize = 10;
const MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_BEFORE_ORIGINAL_LOGS: usize = 64;
const MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_AFTER_ORIGINAL_LOGS: usize = 64;
const MAX_COSTUME_OBJECT_PENDING_TRACE_LOGS: usize = 32;
const MAX_COSTUME_PREVIEW_BRANCH_TRACE_LOGS: usize = 180;
const MAX_COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS: usize = 32;
const MAX_COSTUME_PREVIEW_MODEL_CUSTOM_TRACE_LOGS: usize = 48;
const MAX_COSTUME_PREVIEW_MODEL_SLOT5_TRACE_LOGS: usize = 48;
const MAX_COSTUME_PREVIEW_MODEL_DIFF_ONI_LOGS: usize = 4;
const MAX_COSTUME_PREVIEW_MODEL_DIFF_SLOT5_LOGS: usize = 48;
const MAX_COSTUME_PREVIEW_WIDGET_FIELD_CHANGE_LOGS: usize = 16;
const MAX_LAW_SLOT5_POST_WIDGET_TIMELINE_LOGS: usize = 16;
const MAX_LAW_SLOT5_LATE_REACTIVATE_911_LOGS: usize = 6;
const MAX_LAW_SLOT5_PRE_RENDER_REACTIVATE_911_LOGS: usize = 6;
const MAX_PREVIEW_CHILD_FINAL_LOGS: usize = 8;
const MAX_PREVIEW_CHILD_RENDER_CANDIDATE_LOGS: usize = 6;
const MAX_PREVIEW_CHILD8_TIMELINE_LOGS: usize = 8;
const MAX_PREVIEW_CHILD8_CHANGE_LOGS: usize = 8;
const MAX_PREVIEW_CHILD8_WATCH_LOGS: usize = 8;
const MAX_SLOT5_CHILD8_PREVIEW_UPDATE_LOGS: usize = 6;
const MAX_SLOT5_CHILD8_RESTORE_LOGS: usize = 6;
const MAX_SLOT5_CHILD8_RESTORE_SKIP_LOGS: usize = 6;
const MAX_PREVIEW_CANDIDATE_RESULT_PROBE_LOGS: usize = 8;
const MAX_SLOT5_PREVIEW_READY_FOR_RENDER_LOGS: usize = 6;
const MAX_SLOT5_LATE_READY_REFRESH_LOGS: usize = 6;
const MAX_SLOT5_READY_CHECKPOINT_REFRESH_LOGS: usize = 8;
const MAX_SLOT5_CONTROLLER_STATE_CHANGE_LOGS: usize = 8;
const MAX_SLOT5_911_READY_CHECKPOINT_REFRESH_LOGS: usize = 8;
const MAX_SLOT5_EARLY_CHECKPOINT_911_REFRESH_LOGS: usize = 8;
const MAX_SLOT5_PRE_CONTROLLER_LOSS_WINDOW_LOGS: usize = 16;
const MAX_SLOT5_ORDERING_BOUNDARY_LOGS: usize = 24;
const MAX_SLOT5_PRE_CONTROLLER_LOSS_REFRESH_LOGS: usize = 8;
const MAX_SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS: usize = 8;
const MAX_PREVIEW_RENDER_RESULT_LOGS: usize = 4;
const MAX_PREVIEW_RENDER_ASSET_LOGS: usize = 4;
const MAX_COMPANION_RENDER_RESULT_LOGS: usize = 4;
const MAX_COMPANION_RENDER_ASSET_LOGS: usize = 2;
const MAX_COMPANION_RENDER_MISMATCH_LOGS: usize = 6;
const MAX_COMPANION_RESOURCE_FLOW_LOGS: usize = 8;
const MAX_COMPANION_1957_REACTIVATE_LOGS: usize = 3;
const MAX_COMPANION_MAPPING_STATE_LOGS: usize = 6;
const MAX_SLOT5_COMPANION_MAPPING_RESTORE_LOGS: usize = 4;
const MAX_FINAL_RESULT_KIND_LOGS: usize = 8;
const MAX_SLOT5_UI_LABEL_STATE_LOGS: usize = 10;
const MAX_SLOT5_RENDER_RESULT_DIFF_LOGS: usize = 4;
const MAX_SLOT5_FINAL_RENDER_RESULT_LOGS: usize = 6;
const MAX_SLOT5_FINAL_PREVIEW_STATE_LOGS: usize = 6;
const MAX_ONI_FINAL_RENDER_RESULT_LOGS: usize = 1;
const MAX_SLOT5_RESULT1957_CONSUMER_LOGS: usize = 10;
const MAX_SLOT5_RESULT1957_VERDICT_LOGS: usize = 4;
const MAX_POST_COMPANION_RESULT_STATE_LOGS: usize = 8;
const MAX_SLOT5_POST_COMPANION_RESULT_DIFF_LOGS: usize = 6;
const MAX_POST_COMPANION_INSTANCE_STATE_LOGS: usize = 6;
const MAX_SLOT5_SHARED_RESULT_DRAW_DIFF_LOGS: usize = 4;
const MAX_MAIN_WIDGET_FINAL_LOGS: usize = 6;
const MAX_PREVIEW_MODEL_BINDING_STATE_LOGS: usize = 8;
const MAX_MAIN_WIDGET_CONSUMER_LOGS: usize = 10;
const MAX_LOADER_OBJECT_FINAL_LOGS: usize = 6;
const MAX_MODEL_MANAGER_ALIAS_STATE_LOGS: usize = 6;
const MAX_RENDER_OBJECT_ACTIVATION_DIFF_LOGS: usize = 4;
const MAX_SLOT5_APPLY_READY_STATE_LOGS: usize = 4;
const MAX_SLOT5_CHILD_TOGGLE_STATE_LOGS: usize = 4;
const MAX_LOADER_READY_CHECKPOINT_LOGS: usize = 4;
const MAX_MODEL_MANAGER_ALIAS_CHECKPOINT_LOGS: usize = 4;
const MAX_SLOT5_PENDING_READY_GAP_LOGS: usize = 4;
const MAX_SLOT5_PENDING_CLEARED_AFTER_READY_LOGS: usize = 4;
const MAX_COSTUME_COMPANION_PREVIEW_TRACE_LOGS: usize = 32;
const MAX_COSTUME_PREVIEW_GATE_TRACE_LOGS: usize = 180;
const MAX_COSTUME_LAYOUT_AVAILABILITY_TRACE_LOGS: usize = 260;
const MAX_COSTUME_PREVIEW_RESOURCE_ATTACH_TRACE_LOGS: usize = 48;
const MAX_COSTUME_PREVIEW_RESOURCE_RESOLVE_TRACE_LOGS: usize = 32;
const MAX_COSTUME_PREVIEW_RESOURCE_FLOW_LOGS: usize = 24;
const MAX_COSTUME_PREVIEW_RESOURCE_PATH_LOGS: usize = 12;
const MAX_COSTUME_PREVIEW_RESOURCE_QUEUE_LOGS: usize = 8;
const MAX_COSTUME_PREVIEW_RESOURCE_REACTIVATE_LOGS: usize = 8;
const MAX_COSTUME_PREVIEW_RESOURCE_CHANGE_LOGS: usize = 12;
const MAX_COSTUME_PREVIEW_RESOURCE_BOUNDARY_TRACE_LOGS: usize = 12;
const MAX_COSTUME_PREVIEW_RESOURCE_WATCH_LOGS: usize = 16;
const MAX_COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_LOGS: usize = 0;
const MAX_COSTUME_PREVIEW_MODEL_RESOURCE_STATE_LOGS: usize = 8;
const MAX_UI_CHILD_TOGGLE_TRACE_LOGS: usize = 260;
const MAX_LAW_READY_TIMELINE_LOGS: usize = 120;
const MAX_LAUNCH_COSTUME_STATE_TRACE_LOGS: usize = 128;
const MAX_LAUNCH_COSTUME_STATE_PATCH_LOGS: usize = 32;
const LAUNCH_COSTUME_STATE_UNINTERESTING_TRACE_LOGS: usize = 24;
const MAX_COSTUME_UNLOCK_TRACE_LOGS: usize = 180;
const MAX_COSTUME_VALIDATOR_TRACE_LOGS: usize = 240;
const COSTUME_VALIDATOR_UNINTERESTING_TRACE_LOGS: usize = 40;
const MAX_COSTUME_RESOLVER_TRACE_LOGS: usize = 24;
const COSTUME_RESOLVER_UNINTERESTING_TRACE_LOGS: usize = 40;
const LAW_EXTRA_SLOT_PROBE_FALLBACK_VARIANT_ID: u16 = 587;
const LAW_CUSTOM_LAYOUT_ID: u16 = 699;
const LAW_OFFICIAL_LAYOUT_VARIANTS: [u16; 4] = [57, 58, 555, 586];
const LAW_OFFICIAL_VARIANT_METADATA_EXPECTATIONS: [(u16, u16, u16); 4] = [
    (57, 26, 26),
    (58, 227, 294),
    (555, 272, 26),
    (586, 308, 294),
];
const LAW_EXTRA_SLOT_METADATA_SOURCE_LAYOUT_ID: u16 = 131;
const LAW_EXTRA_SLOT_METADATA_SOURCE_VARIANT_ID: u16 = 586;
const LAW_EXTRA_SLOT_COLOR_VARIATION_SOURCE_VARIANT_ID: u16 = 586;
const LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID: u16 = 586;
const LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID: u32 = 643;
const LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID: u32 = 911;
const LAW_EXTRA_SLOT_SOURCE_DLC_FILE: &str = "DLC_COSTUME_006_586_026_003.bin";
const LAW_EXTRA_SLOT_PRIVATE_MODEL_RAM_CLONE_ENABLED: bool = false;
const LAW_EXTRA_SLOT_PRIVATE_MODEL_MANAGER_ALIAS_ENABLED: bool = false;
const LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID: u16 = 26;
const LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID: u16 = 292;
const LAW_SLOT5_LINKDATA_MODEL_RESOURCE_ID: u16 = 174;
const LAW_SLOT5_PRIVATE_ASSET_DATA_READ_PATCH_ENABLED: bool = false;
#[cfg(test)]
const LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_NAME: &str = "MPLC026_Law";
const LAW_SLOT5_PATCH_VARIANT699_METADATA_RAM_ENABLED: bool = false;
const LAW_SLOT5_PATCH_RUNTIME_UNLOCK_SLOT_LINKDATA_ENABLED: bool = false;
const LAW_SLOT5_PATCH_UI_SLOT_BOUNDS_LINKDATA_ENABLED: bool = false;
const LAW_LINKDATA_ONLY_UNLOCK_VARIANT699_ENABLED: bool = false;
const LAW_EXTRA_SLOT_CUSTOM_MODEL_RESOURCE_PATCH_ENABLED: bool = true;
const LAW_EXTRA_SLOT_COLOR_VARIATION_PATCH_ENABLED: bool = true;
const LAW_EXTRA_SLOT_PREVIEW_MAPPING_DIAGNOSTIC_ENABLED: bool = true;
const RESOURCE_911_PAGE_GUARD_WATCH_ENABLED: bool = false;
const RESOURCE_911_PAGE_GUARD_EVENT_CAP: usize = 64;
const RESOURCE_911_PAGE_GUARD_READ_LOG_CAP: usize = 8;
const RESOURCE_911_PAGE_GUARD_WRITE_LOG_CAP: usize = 16;
const MAX_RESOURCE_911_PAGE_GUARD_LOGS: usize = 96;
const RESOURCE_911_PAGE_GUARD_SINGLE_STEP_ENABLED: bool = true;
const RESOURCE_911_PAGE_GUARD_SINGLE_STEP_CAP: usize = 64;
const RESOURCE_911_DIRECT_RESET_WRITE_LOG_CAP: usize = 8;
const RESOURCE_911_WRITER_BLOCK_LOG_CAP: usize = 0;
const RESOURCE_911_CLEANUP_LOG_CAP: usize = 8;
const RESOURCE_911_REACTIVATION_LOG_CAP: usize = 16;
const RESOURCE_911_WRITER_BLOCK_RING_SIZE: usize = 64;
const RESOURCE_911_WRITER_BLOCK_RING_MASK: usize = RESOURCE_911_WRITER_BLOCK_RING_SIZE - 1;
const RESOURCE_911_WRITER_CODE_DUMP_START_RVA: usize = 0x1582dc0;
const RESOURCE_911_WRITER_CODE_DUMP_LEN: usize = 0x50;
const MAX_COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_LOGS: usize = 16;
const MAX_COSTUME_PREVIEW_PAGE_RESET_CALLSITE_LOGS: usize = 8;
const MAX_COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_LOGS: usize = 8;
const MAX_COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_LOGS: usize = 4;
const LAW_EXTRA_SLOT_SELECTABLE_VARIANT_LIMIT_EXCLUSIVE: u16 = 0x02c0;
const LAW_EXTRA_SLOT_FORCE_UNLOCK_ENABLED: bool = false;
const LAW_EXTRA_SLOT_CUSTOM_NON_DLC_ADMISSION_ENABLED: bool = true;
const LAW_EXTRA_SLOT_FORCE_PREVIEW_VISIBLE_DIAGNOSTIC_ENABLED: bool = false;
const LAW_EXTRA_SLOT_FORCE_CONDITIONAL_PREVIEW_DIAGNOSTIC_ENABLED: bool = false;
const LAW_EXTRA_SLOT_SCENE_LIST_FLAG_DIAGNOSTIC_ENABLED: bool = false;
const LAW_EXTRA_SLOT_FORCE_SCENE_AVAILABLE_DIAGNOSTIC_ENABLED: bool = false;
const LAW_EXTRA_SLOT_CLEAR_SCENE_LOCKED_DIAGNOSTIC_ENABLED: bool = false;
const LAW_EXTRA_SLOT_LAUNCH_PRIVATE_MODEL_ALIAS_ENABLED: bool = false;
const LAW_EXTRA_SLOT_MODEL_READY_OVERRIDE_ENABLED: bool = false;
const LAW_EXTRA_SLOT_LATE_MODEL_READY_OVERRIDE_ENABLED: bool = false;
const LAW_EXTRA_SLOT_MODEL_READY_FLAG_PATCH_ENABLED: bool = false;
const LAW_EXTRA_SLOT_PRIVATE_MODEL_CAN_START_DIAGNOSTIC_ENABLED: bool = false;
const LAW_CUSTOM_SLOT_DORMANT_ASSETS_ALWAYS_ACTIVE: bool = false;
const LINKDATA_ENTRY35_MODEL_ROW_BASE: usize = 0x10;
const LINKDATA_ENTRY35_MODEL_ROW_STRIDE: usize = 0x60;
const LINKDATA_RAM_SCAN_CHUNK_SIZE: usize = 512 * 1024;
const MENU_VISIBLE_ROW_LIST_OFFSET: usize = 0x144;
const MENU_VISIBLE_ROW_COUNT_OFFSET: usize = 0x538;
const MENU_MODE_OFFSET: usize = 0x53c;
const MENU_CATEGORY_MODE_OFFSET: usize = 0x540;
const MENU_CURRENT_ROW_OFFSET: usize = 0xa4;
const MENU_SLOT_INDEX_OFFSET: usize = 0xa8;
const MENU_ROW_OVERRIDE_OFFSET: usize = 0xac;
const MENU_LIST_INDEX_OFFSET: usize = 0x54c;
const MENU_SELECTED_INDEX_OFFSET: usize = 0x550;
const MENU_SELECTED_LAYOUT_OFFSET: usize = 0x554;
const MENU_SELECTED_VARIANT_OFFSET: usize = 0x558;
const COSTUME_SCENE_SELECTED_OBJECT_OFFSET: usize = 0x110;
const COSTUME_SCENE_LIST_BASE_OFFSET: usize = 0x120;
const COSTUME_SCENE_LIST_BANK_STARTS_OFFSET: usize = 0x3e0;
const COSTUME_SCENE_LAYOUT_OFFSET: usize = 0x404;
const COSTUME_SCENE_LIST_BANK_OFFSET: usize = 0x408;
const COSTUME_SCENE_LIST_INDEX_OFFSET: usize = 0x410;
const COSTUME_SCENE_ROW_OFFSET: usize = 0x438;
const COSTUME_SCENE_SELECTED_VARIANT_OFFSET: usize = 0x43c;
const COSTUME_SCENE_MODE_OFFSET: usize = 0x440;
const COSTUME_SCENE_FLAG_REFRESH_OFFSET: usize = 0x447;
const COSTUME_SCENE_FLAG_SPECIAL_OFFSET: usize = 0x449;
const COSTUME_SCENE_FLAG_LOCKED_OFFSET: usize = 0x44d;
const COSTUME_SCENE_SELECTED_OBJECT_LAYOUT_OFFSET: usize = 0x440;
const COSTUME_SCENE_SELECTED_OBJECT_VARIANT_OFFSET: usize = 0x448;
const COSTUME_OBJECT_UPDATE_LAYOUTS_OFFSET: usize = 0x70;
const COSTUME_OBJECT_UPDATE_SLOTS_OFFSET: usize = 0xb0;
const COSTUME_OBJECT_UPDATE_KINDS_OFFSET: usize = 0x130;
const COSTUME_OBJECT_UPDATE_COUNT_OFFSET: usize = 0x170;
const COSTUME_OBJECT_UPDATE_SELECTED_INDEX_OFFSET: usize = 0x174;
const COSTUME_OBJECT_UPDATE_LOAD_ARG0_OFFSET: usize = 0x178;
const COSTUME_OBJECT_UPDATE_LOAD_ARG1_OFFSET: usize = 0x1b8;
const COSTUME_OBJECT_UPDATE_LOAD_ARG2_OFFSET: usize = 0x1f8;
const COSTUME_OBJECT_UPDATE_OBJECT_OFFSET: usize = 0x240;
const COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET: usize = 0x260;
const COSTUME_OBJECT_UPDATE_MODE_OFFSET: usize = 0x444;
const COSTUME_OBJECT_UPDATE_CACHED_LAYOUT_OFFSET: usize = 0x448;
const COSTUME_OBJECT_UPDATE_FLAG_REFRESH_OFFSET: usize = 0x44c;
const COSTUME_OBJECT_UPDATE_FLAG_LOCKED_OFFSET: usize = 0x44d;
const COSTUME_OBJECT_UPDATE_OBJECT_CHILD_OFFSET: usize = 0x58;
const COSTUME_OBJECT_UPDATE_OBJECT_PENDING_OFFSET: usize = 0x2b4;
const COSTUME_OBJECT_UPDATE_CONTROLLER_MODEL_LOADER_OFFSET: usize = 0x40;
const LAUNCH_COSTUME_STATE_MODEL_RESOURCE_OFFSET: usize = 0x1d0;
const COSTUME_SCENE_LIST_BANK_COUNT: usize = 8;
const COSTUME_SCENE_LIST_ENTRY_STRIDE: usize = 0x58;
const COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT: usize = 14;
const COSTUME_SCENE_LIST_ENTRY_FLAGS_OFFSET: usize = 0x3c;
const COSTUME_SCENE_LIST_REBUILD_BANK_STARTS_OFFSET: usize =
    COSTUME_SCENE_LIST_BANK_STARTS_OFFSET - COSTUME_SCENE_LIST_BASE_OFFSET;
const COSTUME_SCENE_LIST_REBUILD_COUNT_OFFSET: usize = 0x2e0;
const COSTUME_SCENE_LIST_REBUILD_SELECTED_LAYOUT_OFFSET: usize = 0x2e4;
const COSTUME_SCENE_LIST_REBUILD_BANK_OFFSET: usize = 0x2e8;
const COSTUME_SCENE_LIST_REBUILD_ROW_OFFSET: usize = 0x2ec;
const COSTUME_SCENE_LIST_REBUILD_INDEX_OFFSET: usize = 0x2f0;
const COSTUME_SCENE_LIST_REBUILD_OWNER_OFFSET: usize = 0x2f8;
const COSTUME_SCENE_LIST_REBUILD_WIDGET_OFFSET: usize = 0x300;
const COSTUME_SCENE_LIST_REBUILD_FLAG_OFFSET: usize = 0x308;
const COSTUME_SCENE_LIST_REBUILD_PARAM2_SOURCE_SLOT_OFFSET: usize = 7 * size_of::<usize>();
const COSTUME_PRIMARY_ROW_STRIDE: usize = 0xdc;
const COSTUME_STATIC_ROOT_OFFSET: usize = 0x18;
const COSTUME_STATIC_LAYOUT_TABLE_OFFSET: usize = 0x08;
const COSTUME_STATIC_ROW_TABLE_OFFSET: usize = 0x28;
const COSTUME_LAYOUT_ROW_COUNT: u32 = 0x2d9;
const COSTUME_LAYOUT_ROW_STRIDE: usize = 0x44;
const COSTUME_LAYOUT_ROW_COPY_SIZE: usize = 0x4b;
const COSTUME_LAYOUT_VARIANTS_OFFSET: usize = 0x28;
const COSTUME_LAYOUT_VARIANT_COUNT: u32 = 0x10;
const COSTUME_LAYOUT_AVAILABILITY_TARGET_OFFSET: usize = 0x14;
const COSTUME_LAYOUT_AVAILABILITY_PREVIEW_OFFSET: usize = 0x16;
const COSTUME_LAYOUT_AVAILABILITY_FLAG_OFFSET: usize = 0x4a;
const COSTUME_LAYOUT_AVAILABILITY_TABLE_OFFSET: usize = 0xc1b4;
const COSTUME_LAYOUT_AVAILABILITY_TABLE_COUNT: usize = 300;
const COSTUME_LAYOUT_AVAILABILITY_TABLE_STRIDE: usize = 2;
const COSTUME_VARIANT_METADATA_BASE_OFFSET: usize = 0xd92c;
const COSTUME_VARIANT_METADATA_STRIDE: usize = 0x1e;
const COSTUME_VARIANT_METADATA_COPY_SIZE: usize = 0x1e;
const COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET: usize = 0x00;
const COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET: usize = 0x06;
const COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE: usize = size_of::<u16>();
const COSTUME_VARIANT_METADATA_COLOR_VARIATION_OFFSET: usize = 0x0c;
const COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE: usize = 0x08;
const COSTUME_VARIANT_METADATA_FLAGS_OFFSET: usize = 0x14;
const COSTUME_VARIANT_METADATA_ENABLED_FLAG: u8 = 0x01;
const COSTUME_VARIANT_METADATA_DLC_ENTITLEMENT_FLAG: u8 = 0x02;
const COSTUME_CATEGORY_COUNT: usize = 0x68;
const COSTUME_RUNTIME_CATEGORY_SLOT_BASE: usize = 10;
const COSTUME_RUNTIME_UNLOCK_SLOT_FLAGS_OFFSET: usize = 0x160;
const COSTUME_RUNTIME_UNLOCK_DIRECT_FLAG: u8 = 0x01;
const LAW_MENU_ROW_ID: usize = 70;
const LAW_MASTER_CATEGORY_ID: i32 = 26;
const LAW_HIDDEN_VARIANT_ID: u32 = 555;
const LAW_EXTRA_SLOT_SOURCE_SLOT_INDEX: usize = 3;
const LAW_MENU_ROW_SLOT_OFFSET: usize = 0xa8;
const LAW_MENU_ROW_SLOT_COUNT: usize = 8;
const LAW_UI_SELECTED_SLOT_LIMIT_RVA: usize = 0x155eca3;
const LAW_UI_SLOT_VARIANT_LIMIT_RVA: usize = 0x1559c9f;
const LAW_UI_SLOT_VISIBILITY_MAX_SLOT_RVA: usize = 0x1559d08;
const LAW_UI_SLOT_VISIBILITY_COUNT_RVA: usize = 0x1559d11;
const LAW_UI_SLOT_LIMIT_BEFORE: u8 = 0x03;
const LAW_UI_SLOT_LIMIT_AFTER: u8 = 0x04;
const LAW_UI_SLOT_VISIBILITY_COUNT_BEFORE: u8 = 0x04;
const LAW_UI_SLOT_VISIBILITY_COUNT_AFTER: u8 = 0x05;
const MODEL_RESOURCE_MANAGER_RVA: usize = 0x1eba7a0;
const MODEL_RESOURCE_STATUS_MANAGER_RVA: usize = 0x1eba7b0;
const GAME_GLOBAL_ACTION_WIDGET_RVA: usize = 0x1ec2b30;
const GAME_GLOBAL_COSTUME_WIDGET_RVA: usize = 0x1ec2b38;
const UI_CHILD_INDEX_START_TABLE_RVA: usize = 0x1e5ec98;
const UI_CHILD_INDEX_START_TABLE_STRIDE: usize = 0x40;
const MODEL_RESOURCE_MANAGER_ENTRY_OFFSET: usize = 0x28;
const MODEL_RESOURCE_MANAGER_ENTRY_STRIDE: usize = 0x20;
const MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET: usize = 0x00;
const MODEL_RESOURCE_MANAGER_ENTRY_STATE_OFFSET: usize = 0x08;
const MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE: usize = 0x20;
const MODEL_RESOURCE_GET_RVA: usize = 0x016ce30;
const MODEL_RESOURCE_GET_STOLEN_LEN: usize = 15;
const MODEL_RESOURCE_LOADED_CHECK_RVA: usize = 0x016ceb0;
const MODEL_RESOURCE_LOADED_CHECK_STOLEN_LEN: usize = 15;
const MODEL_RESOURCE_CAN_START_LOAD_RVA: usize = 0x016cf30;
const MODEL_RESOURCE_CAN_START_LOAD_STOLEN_LEN: usize = 15;
const MODEL_RESOURCE_BUSY_CHECK_RVA: usize = 0x005ad10;
const MODEL_RESOURCE_BUSY_CHECK_STOLEN_LEN: usize = 15;
const MODEL_RESOURCE_ENQUEUE_LOAD_RVA: usize = 0x016dc20;
const MODEL_RESOURCE_ENQUEUE_LOAD_STOLEN_LEN: usize = 21;
const MODEL_RESOURCE_STATUS_CHECK_RVA: usize = 0x016e250;
const MODEL_RESOURCE_STATUS_CHECK_STOLEN_LEN: usize = 15;
const MODEL_LOAD_STATE_STEP_RVA: usize = 0x129e8c0;
const MODEL_LOAD_STATE_STEP_STOLEN_LEN: usize = 14;
const MODEL_READY_WAIT_CHECK_RVA: usize = 0x135af30;
const MODEL_READY_WAIT_CHECK_STOLEN_LEN: usize = 15;
const MODEL_RENDER_ATTACH_RVA: usize = 0x03ce790;
const MODEL_RENDER_ATTACH_STOLEN_LEN: usize = 15;
const MODEL_COLOR_APPLY_RVA: usize = 0x1354510;
const MODEL_COLOR_APPLY_STOLEN_LEN: usize = 19;
const MAX_MODEL_RENDER_ATTACH_LOGS: usize = 96;
const MAX_MODEL_RENDER_ATTACH_GLOBAL_LOGS: usize = 24;
const MAX_MODEL_RENDER_ATTACH_SLOT_CONTEXT_LOGS: usize = 64;
const MAX_MODEL_COLOR_APPLY_LOGS: usize = 160;
const MAX_PREVIEW_ASSET_STATE_LOGS: usize = 96;
const MAX_MODEL_RESOURCE_STATUS_CHECK_LOGS: usize = 220;
const MAX_MODEL_LOAD_STATE_STEP_LOGS: usize = 240;
const MAX_MODEL_READY_WAIT_CHECK_LOGS: usize = 220;
type CreateFileWFn =
    unsafe extern "system" fn(Lpcwstr, Dword, Dword, Lpvoid, Dword, Dword, Handle) -> Handle;
type ReadFileFn = unsafe extern "system" fn(Handle, Lpvoid, Dword, Lpdword, Lpvoid) -> Bool;
type CloseHandleFn = unsafe extern "system" fn(Handle) -> Bool;
type GetFileSizeExFn = unsafe extern "system" fn(Handle, *mut LargeInteger) -> Bool;
type GetFileTimeFn = unsafe extern "system" fn(Handle, Lpvoid, Lpvoid, Lpvoid) -> Bool;
type GetFileTypeFn = unsafe extern "system" fn(Handle) -> Dword;
type SetFilePointerExFn =
    unsafe extern "system" fn(Handle, LargeInteger, *mut LargeInteger, Dword) -> Bool;
type GetFileAttributesWFn = unsafe extern "system" fn(Lpcwstr) -> Dword;
type GetFileAttributesExWFn = unsafe extern "system" fn(Lpcwstr, i32, Lpvoid) -> Bool;
type FindFirstFileWFn = unsafe extern "system" fn(Lpcwstr, *mut Win32FindDataW) -> Handle;
type FindFirstFileExWFn =
    unsafe extern "system" fn(Lpcwstr, i32, Lpvoid, i32, Lpvoid, Dword) -> Handle;
type FindNextFileWFn = unsafe extern "system" fn(Handle, *mut Win32FindDataW) -> Bool;
type FindCloseFn = unsafe extern "system" fn(Handle) -> Bool;
type GetProcAddressFn = unsafe extern "system" fn(Handle, *const c_char) -> *mut c_void;
type SteamApiSteamAppsFn = unsafe extern "system" fn() -> *mut c_void;
type SteamAppsBoolAppIdFn = unsafe extern "system" fn(*mut c_void, u32) -> bool;
type MenuVisibleRowBuilderFn = unsafe extern "system" fn(usize, u32);
type CostumeSelectedHelperFn = unsafe extern "system" fn(usize, u32);
type CostumeRowSlotUiHelperFn = unsafe extern "system" fn(usize, u32);
type CostumeSlotVariantHelperFn = unsafe extern "system" fn(usize, usize, u32, u32);
type CostumeScenePresentationHelperFn = unsafe extern "system" fn(usize, usize, usize);
type CostumeSceneAvailableCheckFn = unsafe extern "system" fn(usize) -> u64;
type CostumeSceneApplyHelperFn = unsafe extern "system" fn(usize);
type CostumeScenePostAvailableTransitionFn = unsafe extern "system" fn(usize);
type CostumeSceneStateRefreshFn = unsafe extern "system" fn(usize);
type CostumeSceneUpdateDispatcherFn = unsafe extern "system" fn(usize);
type CostumeScenePreviewRefreshFn = unsafe extern "system" fn(usize, i32) -> u64;
type CostumeSceneListBuildFn = unsafe extern "system" fn(usize, usize, u32, u32);
type CostumeSceneListRebuildFn = unsafe extern "system" fn(usize, usize, u32, u32, u32, u32);
type CostumeSceneSourceApplyFn = unsafe extern "system" fn(usize, usize);
type CostumeObjectUpdateFn = unsafe extern "system" fn(usize);
type CostumeObjectApplyReadyFn = unsafe extern "system" fn(usize, u32, u32);
type CostumeObjectModelReadyCheckFn = unsafe extern "system" fn(usize, u32, u32, u32) -> u64;
type CostumeObjectRefreshPreviewFn = unsafe extern "system" fn(usize, u32);
type CostumePreviewVisibleBranchFn = unsafe extern "system" fn(usize, u32);
type CostumePreviewHiddenBranchFn = unsafe extern "system" fn(usize, i32);
type CostumePreviewConditionalVisibleBranchFn = unsafe extern "system" fn(usize, u32);
type CostumePreviewModelUpdateFn = unsafe extern "system" fn(usize, u32, i32, i32, u32);
type CostumePreviewWidgetResourceUpdateFn = unsafe extern "system" fn(usize);
type CostumePreviewPageResetFn = unsafe extern "system" fn(usize);
type CostumePreviewResourceCandidateUpdateFn = unsafe extern "system" fn(usize) -> usize;
type CostumeCompanionPreviewUpdateFn = unsafe extern "system" fn(usize, u32, i32, i32) -> usize;
type CostumePreviewGlobalDlcGateFn = unsafe extern "system" fn() -> u64;
type CostumePreviewLayoutModeGateFn = unsafe extern "system" fn(u32, u32) -> u64;
type CostumePreviewTailUpdateFn = unsafe extern "system" fn(usize, u32, i32);
type CostumePreviewResourceAttachFn = unsafe extern "system" fn(usize, u32, u32, i32) -> usize;
type CostumePreviewResourceLookupScanFn =
    unsafe extern "system" fn(usize, usize, usize, usize) -> usize;
type CostumePreviewResourceResolveFn =
    unsafe extern "system" fn(usize, u32, u32, i32, i32) -> usize;
type CostumePreviewResourceResetBoundaryFn = unsafe extern "system" fn(usize);
type UiChildToggleFn = unsafe extern "system" fn(usize, u32, u32) -> usize;
type CostumeSceneFallbackLayoutFn = unsafe extern "system" fn(usize, u32) -> u32;
type CostumeSelectionLookupFn = unsafe extern "system" fn(usize, i32, u32) -> u16;
type CostumeSelectionSetterFn = unsafe extern "system" fn(usize, i32, u16, u32);
type CostumeVariantUnlockCheckFn = unsafe extern "system" fn(u32, u32, u32, i32) -> u64;
type CostumeValidatorFn = unsafe extern "system" fn(u32, i32, i32) -> u64;
type CostumeResolverFn = unsafe extern "system" fn(u32, u32, u32, *mut u32) -> u64;
type LaunchCostumeStateConsumerFn = unsafe extern "system" fn(usize) -> usize;
type ModelResourceGetFn = unsafe extern "system" fn(usize, u32) -> usize;
type ModelResourceCheckFn = unsafe extern "system" fn(usize, u32) -> u64;
type ModelResourceEnqueueLoadFn = unsafe extern "system" fn(usize, u32, usize, usize);
type ModelLoadStateStepFn = unsafe extern "system" fn(usize) -> u64;
type ModelReadyWaitCheckFn = unsafe extern "system" fn(usize) -> u64;
type ModelRenderAttachFn = unsafe extern "system" fn(usize, usize, usize, usize, u32) -> usize;
type ModelColorApplyFn = unsafe extern "system" fn(usize, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum VirtualRuntimeKind {
    Global,
    LawCustomSlot,
    LawCustomOriginal,
}

impl VirtualRuntimeKind {
    fn id(self) -> usize {
        match self {
            Self::Global => 0,
            Self::LawCustomSlot => 1,
            Self::LawCustomOriginal => 2,
        }
    }

    fn from_id(id: usize) -> Option<Self> {
        match id {
            0 => Some(Self::Global),
            1 => Some(Self::LawCustomSlot),
            2 => Some(Self::LawCustomOriginal),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::LawCustomSlot => "law-slot-custom",
            Self::LawCustomOriginal => "law-slot-original",
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FileTime {
    low_date_time: u32,
    high_date_time: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Win32FindDataW {
    file_attributes: Dword,
    creation_time: FileTime,
    last_access_time: FileTime,
    last_write_time: FileTime,
    file_size_high: Dword,
    file_size_low: Dword,
    reserved_0: Dword,
    reserved_1: Dword,
    file_name: [u16; 260],
    alternate_file_name: [u16; 14],
}

#[derive(Debug, Clone, Copy)]
struct ModelResourceSlotMirror {
    source_pointer: usize,
    source_state: u32,
    before_pointer: usize,
    before_state: u32,
    after_pointer: usize,
    after_state: u32,
    write_attempted: bool,
    write_ok: bool,
    after_matches: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelRenderAttachLogScope {
    Global,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CostumeLayoutAvailabilityTrace {
    layout_id: i32,
    row_flag: Option<u8>,
    target_index: Option<u16>,
    preview_value: Option<u16>,
    matched_index: Option<u16>,
    matched_flag: Option<u8>,
    result: u64,
    reason: &'static str,
}

#[link(name = "kernel32")]
extern "system" {
    fn GetSystemTimeAsFileTime(system_time_as_file_time: *mut FileTime);
    fn GetLastError() -> Dword;
    fn RtlCaptureStackBackTrace(
        frames_to_skip: Dword,
        frames_to_capture: Dword,
        back_trace: *mut *mut c_void,
        back_trace_hash: *mut Dword,
    ) -> u16;
}

static ORIGINALS: OnceLock<OriginalFunctions> = OnceLock::new();
static RUNTIME: OnceLock<Mutex<Option<VirtualManager>>> = OnceLock::new();
static LAW_CUSTOM_SLOT_RUNTIME: OnceLock<Mutex<Option<VirtualManager>>> = OnceLock::new();
static LAW_CUSTOM_SLOT_ORIGINAL_RUNTIME: OnceLock<Mutex<Option<VirtualManager>>> = OnceLock::new();
static RDB_TRACKER: OnceLock<Mutex<RdbTracker>> = OnceLock::new();
static FIND_TRACKER: OnceLock<Mutex<HashMap<usize, String>>> = OnceLock::new();
static STACK_CODE_CONTEXTS: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();
static VIRTUAL_SOURCES: OnceLock<Mutex<HashMap<u64, ReplacementSource>>> = OnceLock::new();
static LAW_PRIVATE_MODEL_RAM_PLAN: OnceLock<Mutex<Option<LawPrivateModelRamPlan>>> =
    OnceLock::new();
static LAW_LINKDATA_LAYOUT26_CANDIDATE_RAW: OnceLock<Vec<u8>> = OnceLock::new();
static LAW_LINKDATA_VARIANT699_METADATA_RAW: OnceLock<Vec<u8>> = OnceLock::new();
static LAW_LINKDATA_VARIANT_METADATA_RECORDS: OnceLock<Vec<(u16, Vec<u8>)>> = OnceLock::new();
static LAW_SLOT5_ID699_PREFLIGHT_OK: AtomicBool = AtomicBool::new(false);
static LAW_VARIANT_METADATA_RAM_SELECTED_BASE: AtomicUsize = AtomicUsize::new(0);
static CREATE_FILE_LOGS: AtomicUsize = AtomicUsize::new(0);
static DATA_HIT_LOGS: AtomicUsize = AtomicUsize::new(0);
static DATA_PATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static INDEX_PATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static OPEN_VIRTUAL_LOGS: AtomicUsize = AtomicUsize::new(0);
static VIRTUAL_IO_LOGS: AtomicUsize = AtomicUsize::new(0);
static VERBOSE_IO_LOGS: AtomicBool = AtomicBool::new(false);
static TRACE_DLC_STACKS: AtomicBool = AtomicBool::new(false);
static DLC_STACK_LOGS: AtomicUsize = AtomicUsize::new(0);
static STACK_CODE_CONTEXT_LOGS: AtomicUsize = AtomicUsize::new(0);
static STEAM_PROBE_ENABLED: AtomicBool = AtomicBool::new(false);
static STEAM_APPS_VTABLE_PATCHED: AtomicBool = AtomicBool::new(false);
static STEAM_APPS_V006_DYNAMIC_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static STEAM_APPS_V007_DYNAMIC_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static STEAM_APPS_V008_DYNAMIC_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static STEAM_APPS_B_IS_SUBSCRIBED_APP_DYNAMIC_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static STEAM_APPS_B_IS_DLC_INSTALLED_DYNAMIC_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static STEAM_APPS_B_IS_SUBSCRIBED_APP_VTABLE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static STEAM_APPS_B_IS_DLC_INSTALLED_VTABLE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static STEAM_APPS_LOGS: AtomicUsize = AtomicUsize::new(0);
static DUMP_COSTUME_TABLE_ENABLED: AtomicBool = AtomicBool::new(false);
static COSTUME_TABLE_DUMP_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_TABLE_DUMP_DONE: AtomicBool = AtomicBool::new(false);
static LAW_SLOT_SOURCE_PROBE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAW_SLOT_COUNT_DIFF_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAW_SLOT_BUILDER_FILTER_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAW_SLOT_BUILDER_AFTER_METADATA_PATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAW_LINKDATA_ONLY_RAM_VARIANT699_METADATA_PATCH_DONE: AtomicBool = AtomicBool::new(false);
static LAW_LINKDATA_ONLY_RUNTIME_UNLOCK_SLOT_PATCH_DONE: AtomicBool = AtomicBool::new(false);
static LAW_LINKDATA_ONLY_RUNTIME_UNLOCK_SLOT_PATCH_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static UNLOCK_LAW_HIDDEN_VARIANT_ENABLED: AtomicBool = AtomicBool::new(false);
static DUPLICATE_LAW_VARIANT_SLOT_ENABLED: AtomicBool = AtomicBool::new(false);
static DUPLICATE_LAW_VARIANT_SLOT_DONE: AtomicBool = AtomicBool::new(false);
static LAW_PRIVATE_MODEL_RAM_CLONE_DONE: AtomicBool = AtomicBool::new(false);
static LAW_PRIVATE_MODEL_MANAGER_ALIAS_HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);
static LAW_PRIVATE_MODEL_MANAGER_ALIAS_HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);
static LAW_EXTRA_SLOT_PROBE_VARIANT_ID: AtomicUsize = AtomicUsize::new(0);
static LAW_EXTRA_SLOT_CUSTOM_ACTIVE: AtomicBool = AtomicBool::new(false);
static LAW_PRIVATE_MODEL_READY_LATE_OVERRIDE_ALLOWED: AtomicBool = AtomicBool::new(false);
static TRACE_COSTUME_MENU_ENABLED: AtomicBool = AtomicBool::new(false);
static COSTUME_MENU_HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);
static COSTUME_MENU_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SELECTION_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_IMPORTANT_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_LIST_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_LIST_IMPORTANT_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_LIST_BUILD_DETAIL_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_LIST_REBUILD_DETAIL_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_VISIBLE_DECISION_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_LIST_FLAG_PATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_LOCKED_FLAG_PATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_UPDATE_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_OBJECT_UPDATE_STATE: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_OBJECT_UPDATE_OBJECT: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_PREVIEW_WIDGET_CHILD: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_APPLY_READY_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_MODEL_READY_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_REFRESH_PREVIEW_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_SKIPPED_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_GUARD_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_BEFORE_ORIGINAL_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_AFTER_ORIGINAL_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_PENDING_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_BRANCH_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_MODEL_CUSTOM_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_MODEL_SLOT5_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_MODEL_DIFF_ONI_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_MODEL_DIFF_SLOT5_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_WIDGET_FIELD_CHANGE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAW_SLOT5_POST_WIDGET_TIMELINE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAW_SLOT5_LATE_REACTIVATE_911_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAW_SLOT5_PRE_RENDER_REACTIVATE_911_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_PREVIEW_WIDGET: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_PREVIEW_MODEL_QUEUE: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_PREVIEW_WIDGET_QUEUE: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_PREVIEW_VARIANT: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_PREVIEW_VISIBLE: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_PREVIEW_LAYOUT: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_PREVIEW_FALLBACK: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_PREVIEW_WIDGET_SEQ: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_COMPANION_PREVIEW_WIDGET: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_COMPANION_PREVIEW_LAYOUT: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_COMPANION_PREVIEW_FORCE_REFRESH: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_COMPANION_PREVIEW_SCENE_AVAILABLE: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_COMPANION_PREVIEW_WIDGET_SEQ: AtomicUsize = AtomicUsize::new(0);
static PREVIEW_CHILD_FINAL_LOGS: AtomicUsize = AtomicUsize::new(0);
static PREVIEW_CHILD_RENDER_CANDIDATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static PREVIEW_CHILD8_TIMELINE_LOGS: AtomicUsize = AtomicUsize::new(0);
static PREVIEW_CHILD8_CHANGE_LOGS: AtomicUsize = AtomicUsize::new(0);
static PREVIEW_CHILD8_WATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_CHILD8_PREVIEW_UPDATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_CHILD8_RESTORE_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_CHILD8_RESTORE_SKIP_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_PREVIEW_READY_FOR_RENDER_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_LATE_READY_REFRESH_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_LATE_READY_REFRESH_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static SLOT5_LATE_READY_REPLAYED_WIDGET_SEQ: AtomicUsize = AtomicUsize::new(0);
static SLOT5_LATE_READY_REPLAYED_COMPANION_SEQ: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_SEQ: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE_OBJECT: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_OBJECT: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_LOADER: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER_LOADER: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_ATTACH08: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_TOKEN10: AtomicUsize = AtomicUsize::new(0);
static SLOT5_READY_CHECKPOINT_REFRESH_REPLAYED_SEQ: AtomicUsize = AtomicUsize::new(0);
static SLOT5_911_READY_CHECKPOINT_REFRESH_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_911_READY_CHECKPOINT_REFRESH_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static SLOT5_911_READY_CHECKPOINT_REFRESH_REPLAYED_SEQ: AtomicUsize = AtomicUsize::new(0);
static SLOT5_EARLY_CHECKPOINT_911_REFRESH_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_EARLY_CHECKPOINT_911_REFRESH_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static SLOT5_EARLY_CHECKPOINT_911_REFRESH_REPLAYED_SEQ: AtomicUsize = AtomicUsize::new(0);
static SLOT5_PRE_CONTROLLER_LOSS_WINDOW_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_ORDERING_BOUNDARY_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_CONTROLLER_RESTORE_BRANCH_ABORTED_LOGGED: AtomicBool = AtomicBool::new(false);
static SLOT5_FIRST_WIDGET_BEFORE_CONTROLLER_LOSS_LOGGED: AtomicBool = AtomicBool::new(false);
static SLOT5_WIDGET_AFTER_CONTROLLER_LOSS_LOGGED: AtomicBool = AtomicBool::new(false);
static SLOT5_CONTROLLER_LOST_BEFORE_WIDGET_LOGGED: AtomicBool = AtomicBool::new(false);
static SLOT5_PRE_CONTROLLER_LOSS_REFRESH_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_PRE_CONTROLLER_LOSS_REFRESH_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static SLOT5_PRE_CONTROLLER_LOSS_REFRESH_REPLAYED_SEQ: AtomicUsize = AtomicUsize::new(0);
static SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_REPLAYED_SEQ: AtomicUsize = AtomicUsize::new(0);
static SLOT5_CONTROLLER_STATE_CHANGE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_CONTROLLER_STATE: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_CONTROLLER_LOADER_STATE: AtomicUsize = AtomicUsize::new(0);
static SLOT5_RENDER_CANDIDATE_SEQ: AtomicUsize = AtomicUsize::new(0);
static PREVIEW_RENDER_RESULT_LOGS: AtomicUsize = AtomicUsize::new(0);
static PREVIEW_RENDER_ASSET_LOGS: AtomicUsize = AtomicUsize::new(0);
static COMPANION_RENDER_RESULT_LOGS: AtomicUsize = AtomicUsize::new(0);
static COMPANION_RENDER_ASSET_LOGS: AtomicUsize = AtomicUsize::new(0);
static COMPANION_RENDER_MISMATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static COMPANION_RESOURCE_FLOW_LOGS: AtomicUsize = AtomicUsize::new(0);
static COMPANION_1957_REACTIVATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COMPANION_MAPPING_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_COMPANION_MAPPING_RESTORE_LOGS: AtomicUsize = AtomicUsize::new(0);
static FINAL_RESULT_KIND_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_UI_LABEL_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_RENDER_RESULT_DIFF_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_FINAL_RENDER_RESULT_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_FINAL_PREVIEW_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static ONI_FINAL_RENDER_RESULT_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_RESULT1957_CONSUMER_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_RESULT1957_VERDICT_LOGS: AtomicUsize = AtomicUsize::new(0);
static POST_COMPANION_RESULT_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_POST_COMPANION_RESULT_DIFF_LOGS: AtomicUsize = AtomicUsize::new(0);
static POST_COMPANION_INSTANCE_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_SHARED_RESULT_DRAW_DIFF_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_FINAL_RESULT1957: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_FINAL_RESULT1957_WIDGET: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_FINAL_RENDER_RESULT: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_FINAL_RENDER_RESULT_WIDGET: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_PREVIEW_WIDGET: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_PREVIEW_MODEL_QUEUE: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_PREVIEW_WIDGET_QUEUE: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_PREVIEW_VARIANT: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_PREVIEW_WIDGET_SEQ: AtomicUsize = AtomicUsize::new(0);
static MAIN_WIDGET_FINAL_LOGS: AtomicUsize = AtomicUsize::new(0);
static PREVIEW_MODEL_BINDING_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static MAIN_WIDGET_CONSUMER_LOGS: AtomicUsize = AtomicUsize::new(0);
static LOADER_OBJECT_FINAL_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_MANAGER_ALIAS_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static RENDER_OBJECT_ACTIVATION_DIFF_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_APPLY_READY_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_CHILD_TOGGLE_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LOADER_READY_CHECKPOINT_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_MANAGER_ALIAS_CHECKPOINT_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_PENDING_READY_GAP_LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_PENDING_CLEARED_AFTER_READY_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_MODEL_READY_LOADER: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_MODEL_READY_OBJECT: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_MODEL_READY_SEQ: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_MODEL_READY_LOADER: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_MODEL_READY_OBJECT: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_MODEL_READY_SEQ: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_READY_CHECKPOINT_LOADER: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_READY_CHECKPOINT_OBJECT: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_READY_CHECKPOINT_SEQ: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_READY_CHECKPOINT_PENDING: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_ONI_READY_CHECKPOINT_FLAGS20: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_ONI_READY_CHECKPOINT_STATE28: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_ONI_READY_CHECKPOINT_PHASE2C: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_READY_CHECKPOINT_LOADER: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_READY_CHECKPOINT_OBJECT: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_READY_CHECKPOINT_SEQ: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_READY_CHECKPOINT_PENDING: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_READY_CHECKPOINT_FLAGS20: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_READY_CHECKPOINT_STATE28: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_SLOT5_READY_CHECKPOINT_PHASE2C: AtomicUsize = AtomicUsize::new(usize::MAX);
static SLOT5_CHILD8_WATCH_ARMED: AtomicBool = AtomicBool::new(false);
static SLOT5_CHILD8_WATCH_OBJECT: AtomicUsize = AtomicUsize::new(0);
static SLOT5_CHILD8_WATCH_FLAGS30: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static SLOT5_CHILD8_WATCH_B112: AtomicUsize = AtomicUsize::new(u8::MAX as usize);
static SLOT5_CHILD8_WATCH_V34: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static SLOT5_CHILD8_WATCH_V3C: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static SLOT5_CHILD8_WATCH_V44: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static LAST_ONI_PREVIEW_CHILD8_VALID: AtomicBool = AtomicBool::new(false);
static LAST_ONI_PREVIEW_CHILD8_OBJECT: AtomicUsize = AtomicUsize::new(0);
static LAST_ONI_PREVIEW_CHILD8_FLAGS30: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static LAST_ONI_PREVIEW_CHILD8_B112: AtomicUsize = AtomicUsize::new(u8::MAX as usize);
static LAST_ONI_PREVIEW_CHILD8_V34: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static LAST_ONI_PREVIEW_CHILD8_V3C: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static LAST_ONI_PREVIEW_CHILD8_V44: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static LAST_SLOT5_PREVIEW_CHILD8_VALID: AtomicBool = AtomicBool::new(false);
static LAST_SLOT5_PREVIEW_CHILD8_OBJECT: AtomicUsize = AtomicUsize::new(0);
static LAST_SLOT5_PREVIEW_CHILD8_FLAGS30: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static LAST_SLOT5_PREVIEW_CHILD8_B112: AtomicUsize = AtomicUsize::new(u8::MAX as usize);
static LAST_SLOT5_PREVIEW_CHILD8_V34: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static LAST_SLOT5_PREVIEW_CHILD8_V3C: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static LAST_SLOT5_PREVIEW_CHILD8_V44: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static PREVIEW_CANDIDATE_RESULT_PROBE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_COMPANION_PREVIEW_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_GATE_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_LAYOUT_AVAILABILITY_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_LAYOUT_AVAILABILITY_CHECK_HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);
static COSTUME_PREVIEW_RESOURCE_ATTACH_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_RESOLVE_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_FLOW_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_PATH_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_QUEUE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_REACTIVATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_CHANGE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_BOUNDARY_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_WATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_MODEL_RESOURCE_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_PAGE_RESET_CALLSITE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAUNCH_COSTUME_STATE_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAUNCH_COSTUME_STATE_PATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAW_READY_TIMELINE_SEQ: AtomicUsize = AtomicUsize::new(0);
static LAW_READY_TIMELINE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_UNLOCK_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_VALIDATOR_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static COSTUME_RESOLVER_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static MENU_VISIBLE_ROW_BUILDER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SELECTED_HELPER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_ROW_SLOT_UI_HELPER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SLOT_VARIANT_HELPER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_PRESENTATION_HELPER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_AVAILABLE_CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_APPLY_HELPER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_POST_AVAILABLE_TRANSITION_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_STATE_REFRESH_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_UPDATE_DISPATCHER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_PREVIEW_REFRESH_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_LIST_BUILD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_LIST_REBUILD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_SOURCE_APPLY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_UPDATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_APPLY_READY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_MODEL_READY_CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_REFRESH_PREVIEW_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_VISIBLE_BRANCH_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_HIDDEN_BRANCH_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_MODEL_UPDATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_PAGE_RESET_CALLSITE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_COMPANION_PREVIEW_UPDATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_GLOBAL_DLC_GATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_LAYOUT_MODE_GATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_TAIL_UPDATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_ATTACH_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_RESOLVE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_RESET_BOUNDARY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_BRIDGE: AtomicUsize = AtomicUsize::new(0);
static UI_CHILD_TOGGLE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static UI_CHILD_TOGGLE_TRACE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAUNCH_COSTUME_STATE_CONSUMER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SCENE_FALLBACK_LAYOUT_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SELECTION_LOOKUP_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_SELECTION_SETTER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_VARIANT_UNLOCK_CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_VALIDATOR_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COSTUME_RESOLVER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_GET_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_LOADED_CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_CAN_START_LOAD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_BUSY_CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_ENQUEUE_LOAD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_STATUS_CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_ALIAS_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_SLOT_MIRROR_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_STATUS_CHECK_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_LOAD_STATE_STEP_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_LOAD_STATE_STEP_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_READY_WAIT_CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_READY_WAIT_CHECK_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_RENDER_ATTACH_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RENDER_ATTACH_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_RENDER_ATTACH_GLOBAL_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_RENDER_ATTACH_SLOT_CONTEXT_LOGS: AtomicUsize = AtomicUsize::new(0);
static MODEL_COLOR_APPLY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_COLOR_APPLY_LOGS: AtomicUsize = AtomicUsize::new(0);
static PREVIEW_ASSET_STATE_LOGS: AtomicUsize = AtomicUsize::new(0);
static LAST_LAW_PRIVATE_MODEL_RESOURCE_POINTER: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_OBJECT_BUSY_CHECK_SEQ: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_OBJECT_BUSY_CHECK_LOADER: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_OBJECT_BUSY_CHECK_MODEL: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_OBJECT_BUSY_CHECK_ARG1: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_OBJECT_BUSY_CHECK_ARG2: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_OBJECT_BUSY_CHECK_ORIGINAL_RESULT: AtomicUsize = AtomicUsize::new(0);
static LAST_COSTUME_OBJECT_BUSY_CHECK_RESULT: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_RESOLVE_SEQ: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_RESOLVE_LABEL: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_RESOLVE_RESULT: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_RESOLVE_STATE: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_RESOLVE_FLAGS: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_RESOLVE_MARKER: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_ATTACH_SEQ: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_ATTACH_LABEL: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_ATTACH_RESULT: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_ATTACH_LEN: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_ATTACH_HAS_911: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_TABLE: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_OBSERVED: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_OBSERVED_STATE: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_OBSERVED_FLAGS: AtomicUsize = AtomicUsize::new(0);
static LAST_PREVIEW_RESOURCE_911_OBSERVED_MARKER: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_WATCH_ARMED: AtomicBool = AtomicBool::new(false);
static RESOURCE_911_WATCH_TABLE_BASE: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_WATCH_ENTRY_INDEX: AtomicUsize = AtomicUsize::new(usize::MAX);
static RESOURCE_911_WATCH_ENTRY_PTR: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_WATCH_STATE: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static RESOURCE_911_WATCH_FLAGS: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static RESOURCE_911_WATCH_EXTRA: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static RESOURCE_911_WATCH_MARKER: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static RESOURCE_911_PAGE_GUARD_HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);
static RESOURCE_911_PAGE_GUARD_ACTIVE: AtomicBool = AtomicBool::new(false);
static RESOURCE_911_PAGE_GUARD_PENDING: AtomicBool = AtomicBool::new(false);
static RESOURCE_911_PAGE_GUARD_RELEVANT_HIT: AtomicBool = AtomicBool::new(false);
static RESOURCE_911_PAGE_GUARD_SINGLE_STEP_PENDING: AtomicBool = AtomicBool::new(false);
static RESOURCE_911_PAGE_GUARD_EVENTS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_READ_HITS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_WRITE_HITS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_READ_LOGS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_WRITE_LOGS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_SINGLE_STEP_EVENTS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_LOGS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_PAGE_BASE: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_PAGE_SIZE: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_OLD_PROTECT: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_HIT_RIP: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_HIT_FAULT: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_HIT_OFFSET: AtomicUsize = AtomicUsize::new(usize::MAX);
static RESOURCE_911_PAGE_GUARD_HIT_ACCESS: AtomicUsize = AtomicUsize::new(usize::MAX);
static RESOURCE_911_PAGE_GUARD_HIT_REARMED: AtomicBool = AtomicBool::new(false);
static RESOURCE_911_PAGE_GUARD_HIT_SEQ: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_HIT_TABLE_BASE: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_HIT_ENTRY_INDEX: AtomicUsize = AtomicUsize::new(usize::MAX);
static RESOURCE_911_PAGE_GUARD_HIT_ENTRY_PTR: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_PAGE_GUARD_HIT_STATE: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static RESOURCE_911_PAGE_GUARD_HIT_FLAGS: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static RESOURCE_911_PAGE_GUARD_HIT_EXTRA: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static RESOURCE_911_PAGE_GUARD_HIT_MARKER: AtomicUsize = AtomicUsize::new(u32::MAX as usize);
static RESOURCE_911_DIRECT_RESET_WRITE_LOGS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_WRITER_BLOCK_LOGS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_WRITER_BLOCK_FIRST_WATCH_SEQUENCE_LOGGED: AtomicBool = AtomicBool::new(false);
static RESOURCE_911_CLEANUP_LOGS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_REACTIVATION_LOGS: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_WRITER_BLOCK_SNAPSHOT_SEQ: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_WRITER_BLOCK_SEQUENCE_START: AtomicUsize = AtomicUsize::new(0);
static RESOURCE_911_WRITER_BLOCK_SNAPSHOT_SEQ_RING: [AtomicUsize;
    RESOURCE_911_WRITER_BLOCK_RING_SIZE] =
    [const { AtomicUsize::new(0) }; RESOURCE_911_WRITER_BLOCK_RING_SIZE];
static RESOURCE_911_WRITER_BLOCK_SNAPSHOT_RDI_RING: [AtomicUsize;
    RESOURCE_911_WRITER_BLOCK_RING_SIZE] =
    [const { AtomicUsize::new(0) }; RESOURCE_911_WRITER_BLOCK_RING_SIZE];
static RESOURCE_911_WRITER_BLOCK_SNAPSHOT_RBX_RING: [AtomicUsize;
    RESOURCE_911_WRITER_BLOCK_RING_SIZE] =
    [const { AtomicUsize::new(0) }; RESOURCE_911_WRITER_BLOCK_RING_SIZE];
static RESOURCE_911_WRITER_BLOCK_SNAPSHOT_R12_RING: [AtomicUsize;
    RESOURCE_911_WRITER_BLOCK_RING_SIZE] =
    [const { AtomicUsize::new(0) }; RESOURCE_911_WRITER_BLOCK_RING_SIZE];
static RESOURCE_911_WRITER_CODE_DUMP_LOGGED: AtomicBool = AtomicBool::new(false);
static RESOURCE_911_PAGE_GUARD_HISTORY_SEQ: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_RIP: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_FAULT: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_OFFSET: [AtomicUsize; 8] =
    [const { AtomicUsize::new(usize::MAX) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_ACCESS: [AtomicUsize; 8] =
    [const { AtomicUsize::new(usize::MAX) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_REARMED: [AtomicBool; 8] =
    [const { AtomicBool::new(false) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_STATE: [AtomicUsize; 8] =
    [const { AtomicUsize::new(u32::MAX as usize) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_MARKER: [AtomicUsize; 8] =
    [const { AtomicUsize::new(u32::MAX as usize) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_ID: [AtomicUsize; 8] =
    [const { AtomicUsize::new(u32::MAX as usize) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_Q40: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_RAX: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_RBX: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_RCX: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_RDX: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_RSI: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_RDI: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_R8: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_R9: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_R10: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_PAGE_GUARD_HISTORY_R11: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
static RESOURCE_911_GUARD_MISSED_WRITE_LOGS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MenuStateTrace {
    mode: Option<i32>,
    category_mode: Option<u32>,
    current_row: Option<u32>,
    slot_index: Option<u32>,
    row_override: Option<u32>,
    list_index: Option<i32>,
    visible_count: Option<u32>,
    selected_index: Option<u32>,
    selected_row: Option<u32>,
    selected_layout: Option<u32>,
    selected_variant: Option<u32>,
    focus_variant: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CostumeSceneTrace {
    layout: Option<u32>,
    row: Option<u32>,
    mode: Option<u32>,
    selected_variant: Option<u16>,
    selected_slot: Option<u32>,
    list_bank: Option<u32>,
    list_index: Option<u32>,
    refresh_flag: Option<u8>,
    special_flag: Option<u8>,
    locked_flag: Option<u8>,
    selected_object: Option<usize>,
    object_layout: Option<u32>,
    object_variant: Option<u32>,
    object_slot: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CostumeSceneListRebuildSnapshot {
    count: Option<u32>,
    selected_layout: Option<u32>,
    bank: Option<u32>,
    row: Option<u32>,
    index: Option<u32>,
    owner: Option<usize>,
    widget: Option<usize>,
    flag: Option<u8>,
    param2_source: Option<usize>,
    active_entry: Option<usize>,
    active_layouts: Option<[u32; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT]>,
    active_flags: Option<[u8; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CostumeObjectUpdateTrace {
    count: Option<u32>,
    selected_index: Option<u32>,
    mode: Option<u32>,
    cached_layout: Option<u32>,
    refresh_flag: Option<u8>,
    locked_flag: Option<u8>,
    object: Option<usize>,
    controller: Option<usize>,
    object_child: Option<usize>,
    object_pending: Option<u8>,
    controller_model_loader: Option<usize>,
    selected_layout: Option<u32>,
    selected_slot: Option<u32>,
    selected_kind: Option<u32>,
    selected_load_arg0: Option<u32>,
    selected_load_arg1: Option<u32>,
    selected_load_arg2: Option<u32>,
    selected_variant: Option<u16>,
    selected_model_resource: Option<u16>,
    selected_preview_mapping: Option<u16>,
    selected_preview_mapped_resource: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RefreshPreviewTableProbe {
    preview_arg: u32,
    status: &'static str,
    global: Option<usize>,
    vector: Option<usize>,
    start: Option<usize>,
    end: Option<usize>,
    count: Option<usize>,
    entry: Option<usize>,
    width: Option<u32>,
    height: Option<u32>,
    raw: Option<[u8; COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_STRIDE]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreviewModelWidgetTrace {
    widget: usize,
    child58: Option<usize>,
    field10: Option<u32>,
    mapped294: Option<u32>,
    current_layout2dc: Option<u32>,
    visible2a0: Option<u8>,
    active2a1: Option<u8>,
    child_b4: Option<u32>,
    child_28: Option<usize>,
    child_30: Option<usize>,
    child_38: Option<usize>,
    child_48: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LaunchCostumeStateTrace {
    state: usize,
    index4: Option<u16>,
    flags20: Option<u32>,
    id_1d0: Option<u32>,
    id_1d4: Option<u32>,
    id_1d8: Option<u32>,
    id_1dc: Option<u16>,
    id_1e4: Option<u16>,
    ptr_f0: Option<usize>,
    ptr_f8: Option<usize>,
    ptr_430: Option<usize>,
    ptr_430_7bd8: Option<usize>,
    object_14d: Option<u8>,
    object_1dc: Option<u16>,
    object_232: Option<u8>,
    object_235: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CustomVariantAllocation {
    character: &'static str,
    source_variant_id: u16,
    metadata_source_layout_id: u16,
    metadata_source_variant_id: u16,
    allocated_variant_id: u16,
    slot_index: usize,
    mode: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CustomVariantMetadataClone {
    source_layout_id: u16,
    target_layout_id: u16,
    source_address: usize,
    target_address: usize,
    byte_count: usize,
}

#[derive(Clone, Copy)]
struct OriginalFunctions {
    create_file_w: Option<CreateFileWFn>,
    read_file: Option<ReadFileFn>,
    close_handle: Option<CloseHandleFn>,
    get_file_size_ex: Option<GetFileSizeExFn>,
    get_file_time: Option<GetFileTimeFn>,
    get_file_type: Option<GetFileTypeFn>,
    set_file_pointer_ex: Option<SetFilePointerExFn>,
    get_file_attributes_w: Option<GetFileAttributesWFn>,
    get_file_attributes_ex_w: Option<GetFileAttributesExWFn>,
    find_first_file_w: Option<FindFirstFileWFn>,
    find_first_file_ex_w: Option<FindFirstFileExWFn>,
    find_next_file_w: Option<FindNextFileWFn>,
    find_close: Option<FindCloseFn>,
    get_proc_address: Option<GetProcAddressFn>,
    steam_apps_v006: Option<SteamApiSteamAppsFn>,
    steam_apps_v007: Option<SteamApiSteamAppsFn>,
    steam_apps_v008: Option<SteamApiSteamAppsFn>,
    steam_apps_b_is_subscribed_app: Option<SteamAppsBoolAppIdFn>,
    steam_apps_b_is_dlc_installed: Option<SteamAppsBoolAppIdFn>,
}

impl OriginalFunctions {
    fn empty() -> Self {
        Self {
            create_file_w: None,
            read_file: None,
            close_handle: None,
            get_file_size_ex: None,
            get_file_time: None,
            get_file_type: None,
            set_file_pointer_ex: None,
            get_file_attributes_w: None,
            get_file_attributes_ex_w: None,
            find_first_file_w: None,
            find_first_file_ex_w: None,
            find_next_file_w: None,
            find_close: None,
            get_proc_address: None,
            steam_apps_v006: None,
            steam_apps_v007: None,
            steam_apps_v008: None,
            steam_apps_b_is_subscribed_app: None,
            steam_apps_b_is_dlc_installed: None,
        }
    }
}

#[repr(C)]
struct ImageImportDescriptor {
    original_first_thunk: u32,
    time_date_stamp: u32,
    forwarder_chain: u32,
    name: u32,
    first_thunk: u32,
}

#[derive(Debug, Default)]
struct RdbTracker {
    handles: HashMap<usize, TrackedRdb>,
    total_logs: usize,
}

#[derive(Debug)]
struct TrackedRdb {
    archive_name: String,
    kind: TrackedFileKind,
    read_logs: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackedFileKind {
    Index,
    Data,
}

impl RdbTracker {
    fn track_open(&mut self, handle: Handle, file_name: &str) {
        if handle == INVALID_HANDLE_VALUE {
            return;
        }
        let Some((archive_name, kind)) = tracked_archive_name(file_name) else {
            return;
        };
        self.handles.insert(
            handle as usize,
            TrackedRdb {
                archive_name,
                kind,
                read_logs: 0,
            },
        );
        if verbose_io_logs() {
            log::write_line(format!(
                "Track {} {}: handle=0x{:x}",
                kind.label(),
                self.handles
                    .get(&(handle as usize))
                    .map(|tracked| tracked.archive_name.as_str())
                    .unwrap_or("?"),
                handle as usize
            ));
        }
    }

    fn untrack(&mut self, handle: Handle) {
        if let Some(tracked) = self.handles.remove(&(handle as usize)) {
            if verbose_io_logs() {
                log::write_line(format!(
                    "Close {} {}",
                    tracked.kind.label(),
                    tracked.archive_name
                ));
            }
        }
    }

    fn read_event(&mut self, handle: Handle) -> Option<(TrackedRead, bool)> {
        let tracked = self.handles.get_mut(&(handle as usize))?;
        let should_log = verbose_io_logs() && tracked.read_logs < 64 && self.total_logs < 640;
        if should_log {
            tracked.read_logs += 1;
            self.total_logs += 1;
        }
        Some((
            TrackedRead {
                archive_name: tracked.archive_name.clone(),
                kind: tracked.kind,
            },
            should_log,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrackedRead {
    archive_name: String,
    kind: TrackedFileKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawPrivateModelRamPlan {
    pub source_row_id: u16,
    pub target_row_id: u16,
    pub source_row: Vec<u8>,
    pub validation_rows: Vec<LawPrivateModelValidationRow>,
    pub entry32_name_patch: Option<LawEntry32NamePatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawPrivateModelValidationRow {
    pub row_id: u16,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawEntry32NamePatch {
    pub offset: usize,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub anchors: Vec<LawEntry32Anchor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawEntry32Anchor {
    pub offset: usize,
    pub bytes: Vec<u8>,
}

impl TrackedFileKind {
    fn label(self) -> &'static str {
        match self {
            TrackedFileKind::Index => "RDB",
            TrackedFileKind::Data => "RDB BIN",
        }
    }
}

pub fn publish_replacements(replacements: Vec<VirtualReplacement>) {
    let count = replacements.len();
    let runtime = RUNTIME.get_or_init(|| Mutex::new(None));
    let Ok(mut guard) = runtime.lock() else {
        log::write_line("virtual runtime lock poisoned");
        return;
    };
    *guard = Some(VirtualManager::new(replacements));
    log::write_line(format!("virtual runtime published: {count} replacements"));
}

pub fn publish_law_custom_slot_replacements(replacements: Vec<VirtualReplacement>) {
    let count = replacements.len();
    let runtime = LAW_CUSTOM_SLOT_RUNTIME.get_or_init(|| Mutex::new(None));
    let Ok(mut guard) = runtime.lock() else {
        log::write_line("law-slot-custom virtual runtime lock poisoned");
        return;
    };
    *guard = Some(VirtualManager::new(replacements));
    log::write_line(format!(
        "law-slot-custom virtual runtime published: {count} replacements"
    ));
}

pub fn publish_law_private_model_ram_plan(plan: LawPrivateModelRamPlan) {
    let source = plan.source_row_id;
    let target = plan.target_row_id;
    let validations = plan.validation_rows.len();
    let has_name_patch = plan.entry32_name_patch.is_some();
    let slot = LAW_PRIVATE_MODEL_RAM_PLAN.get_or_init(|| Mutex::new(None));
    let Ok(mut guard) = slot.lock() else {
        log::write_line("Law private model RAM plan lock poisoned");
        return;
    };
    *guard = Some(plan);
    LAW_PRIVATE_MODEL_RAM_CLONE_DONE.store(false, Ordering::Relaxed);
    log::write_line(format!(
        "Law private model RAM plan published source_row={source} target_row={target} validations={validations} name_patch={has_name_patch}"
    ));
}

pub fn publish_law_linkdata_layout26_candidate_raw(raw: &[u8]) {
    let _ = LAW_LINKDATA_LAYOUT26_CANDIDATE_RAW.set(raw.to_vec());
}

fn law_linkdata_layout26_candidate_raw() -> Option<&'static [u8]> {
    LAW_LINKDATA_LAYOUT26_CANDIDATE_RAW.get().map(Vec::as_slice)
}

pub fn publish_law_linkdata_variant699_metadata_raw(raw: &[u8]) {
    let _ = LAW_LINKDATA_VARIANT699_METADATA_RAW.set(raw.to_vec());
}

fn law_linkdata_variant699_metadata_raw() -> Option<&'static [u8]> {
    LAW_LINKDATA_VARIANT699_METADATA_RAW
        .get()
        .map(Vec::as_slice)
}

pub fn publish_law_linkdata_variant_metadata_records(records: &[(u16, Vec<u8>)]) {
    let _ = LAW_LINKDATA_VARIANT_METADATA_RECORDS.set(records.to_vec());
    LAW_VARIANT_METADATA_RAM_SELECTED_BASE.store(0, Ordering::Relaxed);
}

fn law_linkdata_variant_metadata_records() -> Option<&'static [(u16, Vec<u8>)]> {
    LAW_LINKDATA_VARIANT_METADATA_RECORDS
        .get()
        .map(Vec::as_slice)
}

pub fn publish_law_slot5_id699_preflight(ok: bool) {
    LAW_SLOT5_ID699_PREFLIGHT_OK.store(ok, Ordering::Relaxed);
    if ok {
        let module = win::main_module();
        if !module.is_null() {
            maybe_patch_law_duplicate_ui_slot_bounds(module as usize);
        }
    }
}

fn law_slot5_id699_preflight_ok() -> bool {
    LAW_SLOT5_ID699_PREFLIGHT_OK.load(Ordering::Relaxed)
}

pub fn set_verbose_io_logs(enabled: bool) {
    VERBOSE_IO_LOGS.store(enabled, Ordering::Relaxed);
    if enabled {
        log::write_line("verbose IO logs enabled");
    }
}

pub fn set_trace_dlc_stacks(enabled: bool) {
    TRACE_DLC_STACKS.store(enabled, Ordering::Relaxed);
    if enabled {
        log::write_line("DLC/save CreateFileW stack traces enabled");
    }
}

pub fn set_steam_probe_enabled(enabled: bool) {
    STEAM_PROBE_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        log::write_line("SteamApps probe enabled");
    }
}

pub fn set_dump_costume_table(enabled: bool) {
    DUMP_COSTUME_TABLE_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        log::write_line("global costume table dump enabled");
    }
}

pub fn set_duplicate_law_variant_slot(enabled: bool) {
    DUPLICATE_LAW_VARIANT_SLOT_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        log::write_line("Law duplicate variant slot patch enabled");
    }
}

pub fn set_unlock_law_hidden_variant(enabled: bool) {
    UNLOCK_LAW_HIDDEN_VARIANT_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        log::write_line("Law hidden variant unlock patch enabled");
    }
}

pub fn set_trace_costume_menu(enabled: bool) {
    TRACE_COSTUME_MENU_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        log::write_line("costume menu trace enabled");
    }
}

pub fn install_main_module_hooks() {
    let module = win::main_module();
    if module.is_null() {
        log::write_line("hook install skipped: main module not found");
        return;
    }

    let originals = unsafe { install_iat_hooks(module as usize) };
    let installed = count_installed(originals);
    let _ = ORIGINALS.set(originals);
    log::write_line(format!("IAT hooks installed: {installed}"));
    if !INSTALL_INTERNAL_COSTUME_TRACE_HOOKS {
        unsafe { install_law_hidden_variant_unlock_hook(module as usize) };
        unsafe { install_clean_slot_navigation_trace_hooks(module as usize) };
        unsafe { install_clean_costume_object_update_trace_hook(module as usize) };
        maybe_patch_law_duplicate_ui_slot_bounds(module as usize);
        log::write_line("internal costume/model trace hooks disabled in clean LinkData build");
        return;
    }
    maybe_patch_law_duplicate_ui_slot_bounds(module as usize);
    unsafe { install_law_private_model_manager_alias_hooks(module as usize) };
    unsafe { install_costume_menu_hooks(module as usize) };
}

unsafe fn install_law_hidden_variant_unlock_hook(module: usize) {
    if !UNLOCK_LAW_HIDDEN_VARIANT_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if COSTUME_VARIANT_UNLOCK_CHECK_ORIGINAL.load(Ordering::Acquire) != 0 {
        return;
    }
    let installed = install_internal_trace_hook(
        module,
        COSTUME_VARIANT_UNLOCK_CHECK_RVA,
        COSTUME_VARIANT_UNLOCK_CHECK_STOLEN_LEN,
        hooked_costume_variant_unlock_check as usize,
        &COSTUME_VARIANT_UNLOCK_CHECK_ORIGINAL,
        "law hidden variant unlock-check hook",
    );
    if installed {
        log::write_line("Law hidden variant unlock-check hook installed in clean LinkData build");
    }
}

unsafe fn install_clean_slot_navigation_trace_hooks(module: usize) {
    if !INSTALL_CLEAN_SLOT_NAVIGATION_TRACE_HOOKS {
        return;
    }

    let mut installed = 0usize;
    installed += install_internal_trace_hook(
        module,
        MENU_VISIBLE_ROW_BUILDER_RVA,
        MENU_VISIBLE_ROW_BUILDER_STOLEN_LEN,
        hooked_menu_visible_row_builder as usize,
        &MENU_VISIBLE_ROW_BUILDER_ORIGINAL,
        "clean slot navigation visible-row trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SELECTED_HELPER_RVA,
        COSTUME_SELECTED_HELPER_STOLEN_LEN,
        hooked_costume_selected_helper as usize,
        &COSTUME_SELECTED_HELPER_ORIGINAL,
        "clean slot navigation selected-helper trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_ROW_SLOT_UI_HELPER_RVA,
        COSTUME_ROW_SLOT_UI_HELPER_STOLEN_LEN,
        hooked_costume_row_slot_ui_helper as usize,
        &COSTUME_ROW_SLOT_UI_HELPER_ORIGINAL,
        "clean slot navigation row-slot UI trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SLOT_VARIANT_HELPER_RVA,
        COSTUME_SLOT_VARIANT_HELPER_STOLEN_LEN,
        hooked_costume_slot_variant_helper as usize,
        &COSTUME_SLOT_VARIANT_HELPER_ORIGINAL,
        "clean slot navigation slot-variant trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SELECTION_LOOKUP_RVA,
        COSTUME_SELECTION_LOOKUP_STOLEN_LEN,
        hooked_costume_selection_lookup as usize,
        &COSTUME_SELECTION_LOOKUP_ORIGINAL,
        "clean slot navigation selection lookup trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SELECTION_SETTER_RVA,
        COSTUME_SELECTION_SETTER_STOLEN_LEN,
        hooked_costume_selection_setter as usize,
        &COSTUME_SELECTION_SETTER_ORIGINAL,
        "clean slot navigation selection setter trace hook",
    ) as usize;

    log::write_line(format!(
        "clean slot navigation trace hooks installed: {installed}"
    ));
}

unsafe fn install_clean_costume_object_update_trace_hook(module: usize) {
    if !INSTALL_CLEAN_COSTUME_OBJECT_UPDATE_TRACE_HOOK {
        return;
    }
    if COSTUME_OBJECT_UPDATE_ORIGINAL.load(Ordering::Acquire) != 0 {
        return;
    }
    let installed = install_internal_trace_hook(
        module,
        COSTUME_OBJECT_UPDATE_RVA,
        COSTUME_OBJECT_UPDATE_STOLEN_LEN,
        hooked_costume_object_update as usize,
        &COSTUME_OBJECT_UPDATE_ORIGINAL,
        "clean costume object update trace hook",
    );
    if installed {
        log::write_line("clean costume object update trace hook installed");
    }
}

fn maybe_patch_law_duplicate_ui_slot_bounds(module: usize) {
    let runtime_slot_hook_enabled = DUPLICATE_LAW_VARIANT_SLOT_ENABLED.load(Ordering::Relaxed);
    let linkdata_slot5_enabled =
        LAW_SLOT5_PATCH_UI_SLOT_BOUNDS_LINKDATA_ENABLED && law_slot5_id699_preflight_ok();
    if !runtime_slot_hook_enabled && !linkdata_slot5_enabled {
        return;
    }

    patch_single_code_byte(
        module + LAW_UI_SELECTED_SLOT_LIMIT_RVA,
        LAW_UI_SLOT_LIMIT_BEFORE,
        LAW_UI_SLOT_LIMIT_AFTER,
        "Law duplicate UI selected slot bound",
    );
    patch_single_code_byte(
        module + LAW_UI_SLOT_VARIANT_LIMIT_RVA,
        LAW_UI_SLOT_LIMIT_BEFORE,
        LAW_UI_SLOT_LIMIT_AFTER,
        "Law duplicate UI variant slot bound",
    );
    patch_single_code_byte(
        module + LAW_UI_SLOT_VISIBILITY_MAX_SLOT_RVA,
        LAW_UI_SLOT_LIMIT_BEFORE,
        LAW_UI_SLOT_LIMIT_AFTER,
        "Law duplicate UI visibility max slot",
    );
    patch_single_code_byte(
        module + LAW_UI_SLOT_VISIBILITY_COUNT_RVA,
        LAW_UI_SLOT_VISIBILITY_COUNT_BEFORE,
        LAW_UI_SLOT_VISIBILITY_COUNT_AFTER,
        "Law duplicate UI visibility slot count",
    );
    if linkdata_slot5_enabled && !runtime_slot_hook_enabled {
        log::write_line(
            "law-slot5-linkdata-ui-slot-bounds-patch enabled reason=id699_preflight_ok",
        );
    }
}

unsafe fn install_law_private_model_manager_alias_hooks(module: usize) {
    if !LAW_EXTRA_SLOT_PRIVATE_MODEL_MANAGER_ALIAS_ENABLED
        || !DUPLICATE_LAW_VARIANT_SLOT_ENABLED.load(Ordering::Relaxed)
    {
        return;
    }
    if LAW_PRIVATE_MODEL_MANAGER_ALIAS_HOOK_INSTALLED.swap(true, Ordering::Relaxed) {
        return;
    }

    let mut installed = 0;
    installed += install_internal_trace_hook(
        module,
        MODEL_RESOURCE_GET_RVA,
        MODEL_RESOURCE_GET_STOLEN_LEN,
        hooked_model_resource_get as usize,
        &MODEL_RESOURCE_GET_ORIGINAL,
        "model resource get alias hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        MODEL_RESOURCE_LOADED_CHECK_RVA,
        MODEL_RESOURCE_LOADED_CHECK_STOLEN_LEN,
        hooked_model_resource_loaded_check as usize,
        &MODEL_RESOURCE_LOADED_CHECK_ORIGINAL,
        "model resource loaded-check alias hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        MODEL_RESOURCE_CAN_START_LOAD_RVA,
        MODEL_RESOURCE_CAN_START_LOAD_STOLEN_LEN,
        hooked_model_resource_can_start_load as usize,
        &MODEL_RESOURCE_CAN_START_LOAD_ORIGINAL,
        "model resource can-start-load alias hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        MODEL_RESOURCE_BUSY_CHECK_RVA,
        MODEL_RESOURCE_BUSY_CHECK_STOLEN_LEN,
        hooked_model_resource_busy_check as usize,
        &MODEL_RESOURCE_BUSY_CHECK_ORIGINAL,
        "model resource busy-check alias hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        MODEL_RESOURCE_ENQUEUE_LOAD_RVA,
        MODEL_RESOURCE_ENQUEUE_LOAD_STOLEN_LEN,
        hooked_model_resource_enqueue_load as usize,
        &MODEL_RESOURCE_ENQUEUE_LOAD_ORIGINAL,
        "model resource enqueue-load alias hook",
    ) as usize;
    let status_installed = install_internal_trace_hook(
        module,
        MODEL_RESOURCE_STATUS_CHECK_RVA,
        MODEL_RESOURCE_STATUS_CHECK_STOLEN_LEN,
        hooked_model_resource_status_check as usize,
        &MODEL_RESOURCE_STATUS_CHECK_ORIGINAL,
        "model resource status-check trace hook",
    );
    let load_state_installed = install_internal_trace_hook(
        module,
        MODEL_LOAD_STATE_STEP_RVA,
        MODEL_LOAD_STATE_STEP_STOLEN_LEN,
        hooked_model_load_state_step as usize,
        &MODEL_LOAD_STATE_STEP_ORIGINAL,
        "model load state-step trace hook",
    );
    let ready_wait_installed = install_internal_trace_hook(
        module,
        MODEL_READY_WAIT_CHECK_RVA,
        MODEL_READY_WAIT_CHECK_STOLEN_LEN,
        hooked_model_ready_wait_check as usize,
        &MODEL_READY_WAIT_CHECK_ORIGINAL,
        "model ready wait-check trace hook",
    );
    let attach_installed = install_internal_trace_hook(
        module,
        MODEL_RENDER_ATTACH_RVA,
        MODEL_RENDER_ATTACH_STOLEN_LEN,
        hooked_model_render_attach as usize,
        &MODEL_RENDER_ATTACH_ORIGINAL,
        "model render attach trace hook",
    );
    let color_apply_installed = install_internal_trace_hook(
        module,
        MODEL_COLOR_APPLY_RVA,
        MODEL_COLOR_APPLY_STOLEN_LEN,
        hooked_model_color_apply as usize,
        &MODEL_COLOR_APPLY_ORIGINAL,
        "model color-apply trace hook",
    );

    let active = installed == 5;
    LAW_PRIVATE_MODEL_MANAGER_ALIAS_HOOK_ACTIVE.store(active, Ordering::Relaxed);
    log::write_line(format!(
        "Law private model manager alias hooks installed={installed} active={active} status_trace={} load_state_trace={} ready_wait_trace={} attach_trace={} color_apply_trace={} target_resource={} source_resource={}",
        status_installed,
        load_state_installed,
        ready_wait_installed,
        attach_installed,
        color_apply_installed,
        LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID,
        LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID
    ));
    if !active {
        LAW_PRIVATE_MODEL_MANAGER_ALIAS_HOOK_INSTALLED.store(false, Ordering::Relaxed);
    }
}

unsafe fn install_iat_hooks(module: usize) -> OriginalFunctions {
    let mut originals = OriginalFunctions::empty();
    let Some((import_rva, import_size)) = import_directory(module) else {
        log::write_line("hook install skipped: import directory missing");
        return originals;
    };
    if import_rva == 0 || import_size == 0 {
        log::write_line("hook install skipped: empty import directory");
        return originals;
    }

    let mut descriptor = (module + import_rva as usize) as *const ImageImportDescriptor;
    while (*descriptor).name != 0 {
        patch_import_descriptor(module, &*descriptor, &mut originals);
        descriptor = descriptor.add(1);
    }
    originals
}

unsafe fn install_costume_menu_hooks(module: usize) {
    if !TRACE_COSTUME_MENU_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if COSTUME_MENU_HOOK_INSTALLED.swap(true, Ordering::Relaxed) {
        return;
    }

    let mut installed = 0;
    installed += install_internal_trace_hook(
        module,
        MENU_VISIBLE_ROW_BUILDER_RVA,
        MENU_VISIBLE_ROW_BUILDER_STOLEN_LEN,
        hooked_menu_visible_row_builder as usize,
        &MENU_VISIBLE_ROW_BUILDER_ORIGINAL,
        "costume menu visible-row trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SELECTED_HELPER_RVA,
        COSTUME_SELECTED_HELPER_STOLEN_LEN,
        hooked_costume_selected_helper as usize,
        &COSTUME_SELECTED_HELPER_ORIGINAL,
        "costume selected-helper trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_ROW_SLOT_UI_HELPER_RVA,
        COSTUME_ROW_SLOT_UI_HELPER_STOLEN_LEN,
        hooked_costume_row_slot_ui_helper as usize,
        &COSTUME_ROW_SLOT_UI_HELPER_ORIGINAL,
        "costume row-slot UI trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SLOT_VARIANT_HELPER_RVA,
        COSTUME_SLOT_VARIANT_HELPER_STOLEN_LEN,
        hooked_costume_slot_variant_helper as usize,
        &COSTUME_SLOT_VARIANT_HELPER_ORIGINAL,
        "costume slot-variant trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_PRESENTATION_HELPER_RVA,
        COSTUME_SCENE_PRESENTATION_HELPER_STOLEN_LEN,
        hooked_costume_scene_presentation_helper as usize,
        &COSTUME_SCENE_PRESENTATION_HELPER_ORIGINAL,
        "costume scene presentation trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_AVAILABLE_CHECK_RVA,
        COSTUME_SCENE_AVAILABLE_CHECK_STOLEN_LEN,
        hooked_costume_scene_available_check as usize,
        &COSTUME_SCENE_AVAILABLE_CHECK_ORIGINAL,
        "costume scene available-check trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_APPLY_HELPER_RVA,
        COSTUME_SCENE_APPLY_HELPER_STOLEN_LEN,
        hooked_costume_scene_apply_helper as usize,
        &COSTUME_SCENE_APPLY_HELPER_ORIGINAL,
        "costume scene apply trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_POST_AVAILABLE_TRANSITION_RVA,
        COSTUME_SCENE_POST_AVAILABLE_TRANSITION_STOLEN_LEN,
        hooked_costume_scene_post_available_transition as usize,
        &COSTUME_SCENE_POST_AVAILABLE_TRANSITION_ORIGINAL,
        "costume scene post-available transition trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_STATE_REFRESH_RVA,
        COSTUME_SCENE_STATE_REFRESH_STOLEN_LEN,
        hooked_costume_scene_state_refresh as usize,
        &COSTUME_SCENE_STATE_REFRESH_ORIGINAL,
        "costume scene state-refresh trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_UPDATE_DISPATCHER_RVA,
        COSTUME_SCENE_UPDATE_DISPATCHER_STOLEN_LEN,
        hooked_costume_scene_update_dispatcher as usize,
        &COSTUME_SCENE_UPDATE_DISPATCHER_ORIGINAL,
        "costume scene update-dispatcher trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_PREVIEW_REFRESH_RVA,
        COSTUME_SCENE_PREVIEW_REFRESH_STOLEN_LEN,
        hooked_costume_scene_preview_refresh as usize,
        &COSTUME_SCENE_PREVIEW_REFRESH_ORIGINAL,
        "costume scene preview-refresh trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_LIST_BUILD_RVA,
        COSTUME_SCENE_LIST_BUILD_STOLEN_LEN,
        hooked_costume_scene_list_build as usize,
        &COSTUME_SCENE_LIST_BUILD_ORIGINAL,
        "costume scene list-build detail trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_LIST_REBUILD_RVA,
        COSTUME_SCENE_LIST_REBUILD_STOLEN_LEN,
        hooked_costume_scene_list_rebuild as usize,
        &COSTUME_SCENE_LIST_REBUILD_ORIGINAL,
        "costume scene list-rebuild trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_SOURCE_APPLY_RVA,
        COSTUME_SCENE_SOURCE_APPLY_STOLEN_LEN,
        hooked_costume_scene_source_apply as usize,
        &COSTUME_SCENE_SOURCE_APPLY_ORIGINAL,
        "costume scene source-apply trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_OBJECT_UPDATE_RVA,
        COSTUME_OBJECT_UPDATE_STOLEN_LEN,
        hooked_costume_object_update as usize,
        &COSTUME_OBJECT_UPDATE_ORIGINAL,
        "costume object update trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_OBJECT_APPLY_READY_RVA,
        COSTUME_OBJECT_APPLY_READY_STOLEN_LEN,
        hooked_costume_object_apply_ready as usize,
        &COSTUME_OBJECT_APPLY_READY_ORIGINAL,
        "costume object apply-ready trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_OBJECT_MODEL_READY_CHECK_RVA,
        COSTUME_OBJECT_MODEL_READY_CHECK_STOLEN_LEN,
        hooked_costume_object_model_ready_check as usize,
        &COSTUME_OBJECT_MODEL_READY_CHECK_ORIGINAL,
        "costume object busy/pending check trace hook",
    ) as usize;
    if COSTUME_OBJECT_REFRESH_PREVIEW_HOOK_ENABLED {
        installed += install_internal_trace_hook(
            module,
            COSTUME_OBJECT_REFRESH_PREVIEW_RVA,
            COSTUME_OBJECT_REFRESH_PREVIEW_STOLEN_LEN,
            hooked_costume_object_refresh_preview as usize,
            &COSTUME_OBJECT_REFRESH_PREVIEW_ORIGINAL,
            "costume object refresh-preview trace hook",
        ) as usize;
    } else {
        log::write_line(
            "costume object refresh-preview trace hook disabled: generic trampoline cannot relocate RIP-relative prologue",
        );
    }
    installed += install_costume_object_refresh_preview_callsite_trace_hook(module) as usize;
    installed += install_costume_preview_visible_branch_trace_hook(module) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_HIDDEN_BRANCH_RVA,
        COSTUME_PREVIEW_HIDDEN_BRANCH_STOLEN_LEN,
        hooked_costume_preview_hidden_branch as usize,
        &COSTUME_PREVIEW_HIDDEN_BRANCH_ORIGINAL,
        "costume preview hidden-branch trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_RVA,
        COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_STOLEN_LEN,
        hooked_costume_preview_conditional_visible_branch as usize,
        &COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_ORIGINAL,
        "costume preview conditional-visible-branch trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_MODEL_UPDATE_RVA,
        COSTUME_PREVIEW_MODEL_UPDATE_STOLEN_LEN,
        hooked_costume_preview_model_update as usize,
        &COSTUME_PREVIEW_MODEL_UPDATE_ORIGINAL,
        "costume preview model-update trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_RVA,
        COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_STOLEN_LEN,
        hooked_costume_preview_widget_resource_update as usize,
        &COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_ORIGINAL,
        "costume preview widget-resource update trace hook",
    ) as usize;
    installed += install_costume_preview_page_reset_callsite_trace_hook(module) as usize;
    installed += install_costume_preview_page_reset_tailjump_trace_hook(module) as usize;
    installed += install_costume_preview_resource_candidate_update_trace_hook(module) as usize;
    installed += install_costume_companion_preview_update_trace_hook(module) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_GLOBAL_DLC_GATE_RVA,
        COSTUME_PREVIEW_GLOBAL_DLC_GATE_STOLEN_LEN,
        hooked_costume_preview_global_dlc_gate as usize,
        &COSTUME_PREVIEW_GLOBAL_DLC_GATE_ORIGINAL,
        "costume preview global-dlc gate trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_LAYOUT_MODE_GATE_RVA,
        COSTUME_PREVIEW_LAYOUT_MODE_GATE_STOLEN_LEN,
        hooked_costume_preview_layout_mode_gate as usize,
        &COSTUME_PREVIEW_LAYOUT_MODE_GATE_ORIGINAL,
        "costume preview layout-mode gate trace hook",
    ) as usize;
    installed += install_costume_layout_availability_check_trace_hook(module) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_TAIL_UPDATE_RVA,
        COSTUME_PREVIEW_TAIL_UPDATE_STOLEN_LEN,
        hooked_costume_preview_tail_update as usize,
        &COSTUME_PREVIEW_TAIL_UPDATE_ORIGINAL,
        "costume preview tail-update trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_RESOURCE_ATTACH_RVA,
        COSTUME_PREVIEW_RESOURCE_ATTACH_STOLEN_LEN,
        hooked_costume_preview_resource_attach as usize,
        &COSTUME_PREVIEW_RESOURCE_ATTACH_ORIGINAL,
        "costume preview resource-attach trace hook",
    ) as usize;
    installed += install_costume_preview_resource_lookup_scan_trace_hook(module) as usize;
    installed += install_costume_preview_resource_writer_block_trace_hook(module) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_RESOURCE_RESOLVE_RVA,
        COSTUME_PREVIEW_RESOURCE_RESOLVE_STOLEN_LEN,
        hooked_costume_preview_resource_resolve as usize,
        &COSTUME_PREVIEW_RESOURCE_RESOLVE_ORIGINAL,
        "costume preview resource-resolve trace hook",
    ) as usize;
    if COSTUME_PREVIEW_RESOURCE_RESET_BOUNDARY_HOOK_ENABLED {
        installed += install_internal_trace_hook(
            module,
            COSTUME_PREVIEW_RESOURCE_RESET_BOUNDARY_RVA,
            COSTUME_PREVIEW_RESOURCE_RESET_BOUNDARY_STOLEN_LEN,
            hooked_costume_preview_resource_reset_boundary as usize,
            &COSTUME_PREVIEW_RESOURCE_RESET_BOUNDARY_ORIGINAL,
            "costume preview resource reset-boundary trace hook",
        ) as usize;
    } else {
        log::write_line(
            "costume preview resource reset-boundary trace hook disabled: direct hook caused main-menu crash",
        );
    }
    installed +=
        install_costume_preview_resource_virtual_update_callsite_trace_hook(module) as usize;
    installed += install_resource_911_page_guard_watch() as usize;
    installed += install_ui_child_toggle_trace_hook(module) as usize;
    installed += install_internal_trace_hook(
        module,
        LAUNCH_COSTUME_STATE_CONSUMER_RVA,
        LAUNCH_COSTUME_STATE_CONSUMER_STOLEN_LEN,
        hooked_launch_costume_state_consumer as usize,
        &LAUNCH_COSTUME_STATE_CONSUMER_ORIGINAL,
        "launch costume state-consumer trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SCENE_FALLBACK_LAYOUT_RVA,
        COSTUME_SCENE_FALLBACK_LAYOUT_STOLEN_LEN,
        hooked_costume_scene_fallback_layout as usize,
        &COSTUME_SCENE_FALLBACK_LAYOUT_ORIGINAL,
        "costume scene fallback-layout trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SELECTION_LOOKUP_RVA,
        COSTUME_SELECTION_LOOKUP_STOLEN_LEN,
        hooked_costume_selection_lookup as usize,
        &COSTUME_SELECTION_LOOKUP_ORIGINAL,
        "costume selection lookup trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_SELECTION_SETTER_RVA,
        COSTUME_SELECTION_SETTER_STOLEN_LEN,
        hooked_costume_selection_setter as usize,
        &COSTUME_SELECTION_SETTER_ORIGINAL,
        "costume selection setter trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_VARIANT_UNLOCK_CHECK_RVA,
        COSTUME_VARIANT_UNLOCK_CHECK_STOLEN_LEN,
        hooked_costume_variant_unlock_check as usize,
        &COSTUME_VARIANT_UNLOCK_CHECK_ORIGINAL,
        "costume variant unlock-check trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_VALIDATOR_RVA,
        COSTUME_VALIDATOR_STOLEN_LEN,
        hooked_costume_validator as usize,
        &COSTUME_VALIDATOR_ORIGINAL,
        "costume validator trace hook",
    ) as usize;
    installed += install_internal_trace_hook(
        module,
        COSTUME_RESOLVER_RVA,
        COSTUME_RESOLVER_STOLEN_LEN,
        hooked_costume_resolver as usize,
        &COSTUME_RESOLVER_ORIGINAL,
        "costume resolver trace hook",
    ) as usize;

    if installed == 0 {
        COSTUME_MENU_HOOK_INSTALLED.store(false, Ordering::Relaxed);
    }
}

unsafe fn install_internal_trace_hook(
    module: usize,
    rva: usize,
    stolen_len: usize,
    replacement: usize,
    original_storage: &'static AtomicUsize,
    label: &str,
) -> bool {
    let target = module + rva;
    let Some(trampoline) = create_absolute_jump_trampoline(target, stolen_len) else {
        log::write_line(format!(
            "{label} failed target=game+0x{rva:x} error=trampoline_failed"
        ));
        return false;
    };
    original_storage.store(trampoline, Ordering::Release);

    if !patch_absolute_jump(target, replacement, stolen_len) {
        original_storage.store(0, Ordering::Release);
        log::write_line(format!(
            "{label} failed target=game+0x{rva:x} error=patch_failed"
        ));
        return false;
    }

    log::write_line(format!(
        "{label} installed target=game+0x{rva:x} trampoline=0x{trampoline:x}"
    ));
    true
}

unsafe fn install_costume_object_refresh_preview_callsite_trace_hook(module: usize) -> bool {
    if !COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_HOOK_ENABLED {
        log::write_line("costume object refresh-preview callsite trace hook disabled");
        return false;
    }
    if COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_NEXT_CALLSITE_ENABLED
        || COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_PREV_CALLSITE_ENABLED
    {
        log::write_line(format!(
            "costume object refresh-preview secondary callsites prepared next=game+0x{:x} prev=game+0x{:x} enabled_next={} enabled_prev={}",
            COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_NEXT_CALLSITE_RVA,
            COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_PREV_CALLSITE_RVA,
            COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_NEXT_CALLSITE_ENABLED,
            COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_PREV_CALLSITE_ENABLED
        ));
    }

    let callsite = module + COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA;
    let mut original = [0u8; COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_LEN];
    if !read_exact_process_memory(callsite, &mut original) {
        log::write_line(format!(
            "costume object refresh-preview callsite trace hook failed target=game+0x{:x} error=read_failed",
            COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA
        ));
        return false;
    }
    if original != COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_BYTES {
        log::write_line(format!(
            "costume object refresh-preview callsite trace hook skipped target=game+0x{:x} error=unexpected_call bytes={}",
            COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA,
            format_bytes(&original)
        ));
        return false;
    }

    let original_target = module + COSTUME_OBJECT_REFRESH_PREVIEW_RVA;
    COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.store(original_target, Ordering::Release);
    let Some(bridge) = create_near_absolute_jump_bridge(
        callsite,
        hooked_costume_object_refresh_preview_callsite as usize,
    ) else {
        COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume object refresh-preview callsite trace hook failed target=game+0x{:x} error=bridge_failed",
            COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA
        ));
        return false;
    };

    let Some(patch) = relative_call_bytes(callsite, bridge) else {
        COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume object refresh-preview callsite trace hook failed target=game+0x{:x} bridge=0x{bridge:x} error=bridge_out_of_range",
            COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA
        ));
        return false;
    };
    if !patch_exact_bytes(callsite, &patch) {
        COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume object refresh-preview callsite trace hook failed target=game+0x{:x} bridge=0x{bridge:x} error=patch_failed",
            COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA
        ));
        return false;
    }

    log::write_line(format!(
        "costume object refresh-preview callsite trace hook installed target=game+0x{:x} original=game+0x{:x} bridge=0x{bridge:x}",
        COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA,
        COSTUME_OBJECT_REFRESH_PREVIEW_RVA
    ));
    true
}

unsafe fn install_costume_preview_page_reset_callsite_trace_hook(module: usize) -> bool {
    if !COSTUME_PREVIEW_PAGE_RESET_CALLSITE_HOOK_ENABLED {
        log::write_line("costume preview page-reset callsite trace hook disabled");
        return false;
    }

    let callsite = module + COSTUME_PREVIEW_PAGE_RESET_CALLSITE_RVA;
    let mut original = [0u8; COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_LEN];
    if !read_exact_process_memory(callsite, &mut original) {
        log::write_line(format!(
            "costume preview page-reset callsite trace hook failed target=game+0x{:x} error=read_failed",
            COSTUME_PREVIEW_PAGE_RESET_CALLSITE_RVA
        ));
        return false;
    }
    if original != COSTUME_PREVIEW_PAGE_RESET_CALLSITE_BYTES {
        log::write_line(format!(
            "costume preview page-reset callsite trace hook skipped target=game+0x{:x} error=unexpected_call bytes={}",
            COSTUME_PREVIEW_PAGE_RESET_CALLSITE_RVA,
            format_bytes(&original)
        ));
        return false;
    }

    let original_target = module + COSTUME_PREVIEW_PAGE_RESET_CALLSITE_TARGET_RVA;
    COSTUME_PREVIEW_PAGE_RESET_CALLSITE_ORIGINAL.store(original_target, Ordering::Release);
    let Some(bridge) = create_near_absolute_jump_bridge(
        callsite,
        hooked_costume_preview_page_reset_callsite as usize,
    ) else {
        COSTUME_PREVIEW_PAGE_RESET_CALLSITE_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume preview page-reset callsite trace hook failed target=game+0x{:x} error=bridge_failed",
            COSTUME_PREVIEW_PAGE_RESET_CALLSITE_RVA
        ));
        return false;
    };

    let Some(patch) = relative_call_bytes(callsite, bridge) else {
        COSTUME_PREVIEW_PAGE_RESET_CALLSITE_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume preview page-reset callsite trace hook failed target=game+0x{:x} bridge=0x{bridge:x} error=bridge_out_of_range",
            COSTUME_PREVIEW_PAGE_RESET_CALLSITE_RVA
        ));
        return false;
    };
    if !patch_exact_bytes(callsite, &patch) {
        COSTUME_PREVIEW_PAGE_RESET_CALLSITE_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume preview page-reset callsite trace hook failed target=game+0x{:x} bridge=0x{bridge:x} error=patch_failed",
            COSTUME_PREVIEW_PAGE_RESET_CALLSITE_RVA
        ));
        return false;
    }

    log::write_line(format!(
        "costume preview page-reset callsite trace hook installed target=game+0x{:x} original=game+0x{:x} bridge=0x{bridge:x}",
        COSTUME_PREVIEW_PAGE_RESET_CALLSITE_RVA,
        COSTUME_PREVIEW_PAGE_RESET_CALLSITE_TARGET_RVA
    ));
    true
}

unsafe fn install_costume_preview_page_reset_tailjump_trace_hook(module: usize) -> bool {
    if !COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_HOOK_ENABLED {
        log::write_line("costume preview page-reset tailjump trace hook disabled");
        return false;
    }

    let tailjump = module + COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_RVA;
    let mut original = [0u8; COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_LEN];
    if !read_exact_process_memory(tailjump, &mut original) {
        log::write_line(format!(
            "costume preview page-reset tailjump trace hook failed target=game+0x{:x} error=read_failed",
            COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_RVA
        ));
        return false;
    }
    if original != COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_BYTES {
        log::write_line(format!(
            "costume preview page-reset tailjump trace hook skipped target=game+0x{:x} error=unexpected_jmp bytes={}",
            COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_RVA,
            format_bytes(&original)
        ));
        return false;
    }

    let original_target = module + COSTUME_PREVIEW_PAGE_RESET_CALLSITE_TARGET_RVA;
    COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_ORIGINAL.store(original_target, Ordering::Release);
    let Some(bridge) = create_near_absolute_jump_bridge(
        tailjump,
        hooked_costume_preview_page_reset_tailjump as usize,
    ) else {
        COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume preview page-reset tailjump trace hook failed target=game+0x{:x} error=bridge_failed",
            COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_RVA
        ));
        return false;
    };

    let Some(patch) = relative_jump_bytes(tailjump, bridge) else {
        COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume preview page-reset tailjump trace hook failed target=game+0x{:x} bridge=0x{bridge:x} error=bridge_out_of_range",
            COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_RVA
        ));
        return false;
    };
    if !patch_exact_bytes(tailjump, &patch) {
        COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume preview page-reset tailjump trace hook failed target=game+0x{:x} bridge=0x{bridge:x} error=patch_failed",
            COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_RVA
        ));
        return false;
    }

    log::write_line(format!(
        "costume preview page-reset tailjump trace hook installed target=game+0x{:x} original=game+0x{:x} bridge=0x{bridge:x}",
        COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_RVA,
        COSTUME_PREVIEW_PAGE_RESET_CALLSITE_TARGET_RVA
    ));
    true
}

unsafe fn install_costume_preview_resource_candidate_update_trace_hook(module: usize) -> bool {
    if !COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_HOOK_ENABLED {
        log::write_line("costume preview resource candidate-update trace hook disabled");
        return false;
    }

    let target = module + COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_RVA;
    let mut original = [0u8; COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_STOLEN_LEN];
    if !read_exact_process_memory(target, &mut original) {
        log::write_line(format!(
            "costume preview resource candidate-update trace hook failed target=game+0x{:x} error=read_failed",
            COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_RVA
        ));
        return false;
    }
    if original != COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_BYTES {
        log::write_line(format!(
            "costume preview resource candidate-update trace hook skipped target=game+0x{:x} error=unexpected_prologue bytes={}",
            COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_RVA,
            format_bytes(&original)
        ));
        return false;
    }

    install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_RVA,
        COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_STOLEN_LEN,
        hooked_costume_preview_resource_candidate_update as usize,
        &COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_ORIGINAL,
        "costume preview resource candidate-update trace hook",
    )
}

unsafe fn install_costume_preview_resource_virtual_update_callsite_trace_hook(
    module: usize,
) -> bool {
    if !COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_HOOK_ENABLED {
        log::write_line("costume preview resource virtual-update callsite trace hook disabled");
        return false;
    }

    let callsite = module + COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RVA;
    let resume = module + COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RESUME_RVA;
    let mut original = [0u8; COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_LEN];
    if !read_exact_process_memory(callsite, &mut original) {
        log::write_line(format!(
            "costume preview resource virtual-update callsite trace hook failed target=game+0x{:x} error=read_failed",
            COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RVA
        ));
        return false;
    }
    if original != COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_BYTES {
        log::write_line(format!(
            "costume preview resource virtual-update callsite trace hook skipped target=game+0x{:x} error=unexpected_call bytes={}",
            COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RVA,
            format_bytes(&original)
        ));
        return false;
    }

    let Some(bridge) = create_costume_preview_resource_virtual_update_callsite_bridge(
        callsite,
        resume,
        hooked_costume_preview_resource_virtual_update_callsite_before as usize,
        hooked_costume_preview_resource_virtual_update_callsite_after as usize,
    ) else {
        log::write_line(format!(
            "costume preview resource virtual-update callsite trace hook failed target=game+0x{:x} error=bridge_failed",
            COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RVA
        ));
        return false;
    };

    let Some(patch) = relative_call_bytes(callsite, bridge) else {
        COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_BRIDGE.store(0, Ordering::Release);
        log::write_line(format!(
            "costume preview resource virtual-update callsite trace hook failed target=game+0x{:x} bridge=0x{bridge:x} error=bridge_out_of_range",
            COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RVA
        ));
        return false;
    };
    if !patch_exact_bytes(callsite, &patch) {
        COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_BRIDGE.store(0, Ordering::Release);
        log::write_line(format!(
            "costume preview resource virtual-update callsite trace hook failed target=game+0x{:x} bridge=0x{bridge:x} error=patch_failed",
            COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RVA
        ));
        return false;
    }

    COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_BRIDGE.store(bridge, Ordering::Release);
    log::write_line(format!(
        "costume preview resource virtual-update callsite trace hook installed target=game+0x{:x} resume=game+0x{:x} bridge=0x{bridge:x}",
        COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RVA,
        COSTUME_PREVIEW_RESOURCE_VIRTUAL_UPDATE_CALLSITE_RESUME_RVA
    ));
    true
}

unsafe fn install_costume_preview_resource_lookup_scan_trace_hook(module: usize) -> bool {
    if !COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_HOOK_ENABLED {
        log::write_line("costume preview resource lookup-scan trace hook disabled");
        return false;
    }

    let target = module + COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_RVA;
    let mut original = [0u8; COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_STOLEN_LEN];
    if !read_exact_process_memory(target, &mut original) {
        log::write_line(format!(
            "costume preview resource lookup-scan trace hook failed target=game+0x{:x} error=read_failed",
            COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_RVA
        ));
        return false;
    }
    if original != COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_BYTES {
        log::write_line(format!(
            "costume preview resource lookup-scan trace hook skipped target=game+0x{:x} error=unexpected_prologue bytes={}",
            COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_RVA,
            format_bytes(&original)
        ));
        return false;
    }

    install_internal_trace_hook(
        module,
        COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_RVA,
        COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_STOLEN_LEN,
        hooked_costume_preview_resource_lookup_scan as usize,
        &COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_ORIGINAL,
        "costume preview resource lookup-scan trace hook",
    )
}

unsafe fn install_costume_preview_resource_writer_block_trace_hook(module: usize) -> bool {
    if !COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_HOOK_ENABLED
        && !COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_SNAPSHOT_HOOK_ENABLED
    {
        log::write_line("costume preview resource writer-block trace hook disabled");
        return false;
    }

    let target = module + COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RVA;
    let mut original = [0u8; COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_STOLEN_LEN];
    if !read_exact_process_memory(target, &mut original) {
        log::write_line(format!(
            "costume preview resource writer-block trace hook failed target=game+0x{:x} error=read_failed",
            COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RVA
        ));
        return false;
    }
    if original != COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_BYTES {
        log::write_line(format!(
            "costume preview resource writer-block trace hook skipped target=game+0x{:x} error=unexpected_bytes bytes={}",
            COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RVA,
            format_bytes(&original)
        ));
        return false;
    }

    let resume = module + COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RESUME_RVA;
    let bridge = if COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_SNAPSHOT_HOOK_ENABLED {
        create_costume_preview_resource_writer_block_snapshot_bridge(target, resume)
    } else {
        create_costume_preview_resource_writer_block_bridge(
            target,
            resume,
            hooked_costume_preview_resource_writer_block as usize,
        )
    };
    let Some(bridge) = bridge else {
        log::write_line(format!(
            "costume preview resource writer-block trace hook failed target=game+0x{:x} error=bridge_failed",
            COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RVA
        ));
        return false;
    };

    if !patch_absolute_jump(
        target,
        bridge,
        COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_STOLEN_LEN,
    ) {
        log::write_line(format!(
            "costume preview resource writer-block trace hook failed target=game+0x{:x} bridge=0x{bridge:x} error=patch_failed",
            COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RVA
        ));
        return false;
    }

    log::write_line(format!(
        "costume preview resource writer-block {} hook installed target=game+0x{:x} resume=game+0x{:x} bridge=0x{bridge:x}",
        if COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_SNAPSHOT_HOOK_ENABLED {
            "snapshot"
        } else {
            "trace"
        },
        COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RVA,
        COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RESUME_RVA
    ));
    true
}

unsafe fn install_costume_preview_visible_branch_trace_hook(module: usize) -> bool {
    let rva = COSTUME_PREVIEW_VISIBLE_BRANCH_RVA;
    let target = module + rva;
    let Some(trampoline) = create_costume_preview_visible_branch_trampoline(module, target) else {
        log::write_line(format!(
            "costume preview visible-branch trace hook failed target=game+0x{rva:x} error=trampoline_failed"
        ));
        return false;
    };
    COSTUME_PREVIEW_VISIBLE_BRANCH_ORIGINAL.store(trampoline, Ordering::Release);

    if !patch_absolute_jump(
        target,
        hooked_costume_preview_visible_branch as usize,
        COSTUME_PREVIEW_VISIBLE_BRANCH_STOLEN_LEN,
    ) {
        COSTUME_PREVIEW_VISIBLE_BRANCH_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume preview visible-branch trace hook failed target=game+0x{rva:x} error=patch_failed"
        ));
        return false;
    }

    log::write_line(format!(
        "costume preview visible-branch trace hook installed target=game+0x{rva:x} trampoline=0x{trampoline:x}"
    ));
    true
}

unsafe fn install_costume_companion_preview_update_trace_hook(module: usize) -> bool {
    let rva = COSTUME_COMPANION_PREVIEW_UPDATE_RVA;
    let target = module + rva;
    let Some(trampoline) = create_costume_companion_preview_update_trampoline(module, target)
    else {
        log::write_line(format!(
            "costume companion preview-update trace hook failed target=game+0x{rva:x} error=trampoline_failed"
        ));
        return false;
    };
    COSTUME_COMPANION_PREVIEW_UPDATE_ORIGINAL.store(trampoline, Ordering::Release);

    if !patch_absolute_jump(
        target,
        hooked_costume_companion_preview_update as usize,
        COSTUME_COMPANION_PREVIEW_UPDATE_STOLEN_LEN,
    ) {
        COSTUME_COMPANION_PREVIEW_UPDATE_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "costume companion preview-update trace hook failed target=game+0x{rva:x} error=patch_failed"
        ));
        return false;
    }

    log::write_line(format!(
        "costume companion preview-update trace hook installed target=game+0x{rva:x} trampoline=0x{trampoline:x}"
    ));
    true
}

unsafe fn install_costume_layout_availability_check_trace_hook(module: usize) -> bool {
    let rva = COSTUME_LAYOUT_AVAILABILITY_CHECK_RVA;
    let target = module + rva;
    if COSTUME_LAYOUT_AVAILABILITY_CHECK_HOOK_INSTALLED.swap(true, Ordering::Relaxed) {
        return false;
    }

    let mut original = [0u8; COSTUME_LAYOUT_AVAILABILITY_CHECK_STOLEN_LEN];
    if !read_exact_process_memory(target, &mut original) {
        COSTUME_LAYOUT_AVAILABILITY_CHECK_HOOK_INSTALLED.store(false, Ordering::Relaxed);
        log::write_line(format!(
            "costume layout availability-check trace hook failed target=game+0x{rva:x} error=read_failed"
        ));
        return false;
    }

    let expected = [
        0x48, 0x8b, 0x05, 0x71, 0x14, 0xbc, 0x00, 0x48, 0x8b, 0x50, 0x18, 0x48, 0x63, 0xc1,
    ];
    if original != expected {
        COSTUME_LAYOUT_AVAILABILITY_CHECK_HOOK_INSTALLED.store(false, Ordering::Relaxed);
        log::write_line(format!(
            "costume layout availability-check trace hook skipped target=game+0x{rva:x} error=unexpected_prologue bytes={}",
            format_bytes(&original)
        ));
        return false;
    }

    if !patch_absolute_jump(
        target,
        hooked_costume_layout_availability_check as usize,
        COSTUME_LAYOUT_AVAILABILITY_CHECK_STOLEN_LEN,
    ) {
        COSTUME_LAYOUT_AVAILABILITY_CHECK_HOOK_INSTALLED.store(false, Ordering::Relaxed);
        log::write_line(format!(
            "costume layout availability-check trace hook failed target=game+0x{rva:x} error=patch_failed"
        ));
        return false;
    }

    log::write_line(format!(
        "costume layout availability-check trace hook installed target=game+0x{rva:x} no_trampoline=true"
    ));
    true
}

unsafe fn install_ui_child_toggle_trace_hook(module: usize) -> bool {
    let target = module + UI_CHILD_TOGGLE_RVA;
    let Some(trampoline) = create_ui_child_toggle_trampoline(target) else {
        log::write_line(format!(
            "ui child toggle trace hook failed target=game+0x{:x} error=trampoline_failed",
            UI_CHILD_TOGGLE_RVA
        ));
        return false;
    };
    UI_CHILD_TOGGLE_ORIGINAL.store(trampoline, Ordering::Release);
    if !patch_absolute_jump(
        target,
        hooked_ui_child_toggle as usize,
        UI_CHILD_TOGGLE_STOLEN_LEN,
    ) {
        UI_CHILD_TOGGLE_ORIGINAL.store(0, Ordering::Release);
        log::write_line(format!(
            "ui child toggle trace hook failed target=game+0x{:x} error=patch_failed",
            UI_CHILD_TOGGLE_RVA
        ));
        return false;
    }
    log::write_line(format!(
        "ui child toggle trace hook installed target=game+0x{:x}",
        UI_CHILD_TOGGLE_RVA
    ));
    true
}

unsafe fn create_absolute_jump_trampoline(target: usize, stolen_len: usize) -> Option<usize> {
    if stolen_len < ABSOLUTE_JUMP_LEN {
        return None;
    }

    let mut original = vec![0u8; stolen_len];
    if !read_exact_process_memory(target, &mut original) {
        return None;
    }

    let trampoline_size = stolen_len + ABSOLUTE_JUMP_LEN;
    let trampoline = win::allocate_executable_memory(trampoline_size)?;
    std::ptr::copy_nonoverlapping(original.as_ptr(), trampoline as *mut u8, stolen_len);
    let jump_back = absolute_jump_bytes(target + stolen_len);
    std::ptr::copy_nonoverlapping(
        jump_back.as_ptr(),
        (trampoline + stolen_len) as *mut u8,
        jump_back.len(),
    );
    let _ = win::flush_instruction_cache(trampoline, trampoline_size);
    Some(trampoline)
}

unsafe fn create_near_absolute_jump_bridge(anchor: usize, destination: usize) -> Option<usize> {
    let bridge = win::allocate_executable_memory_near(anchor, ABSOLUTE_JUMP_LEN)?;
    let jump = absolute_jump_bytes(destination);
    std::ptr::copy_nonoverlapping(jump.as_ptr(), bridge as *mut u8, jump.len());
    let _ = win::flush_instruction_cache(bridge, jump.len());
    Some(bridge)
}

unsafe fn create_costume_preview_resource_virtual_update_callsite_bridge(
    anchor: usize,
    resume: usize,
    before_hook: usize,
    after_hook: usize,
) -> Option<usize> {
    let mut code = Vec::with_capacity(180);
    code.extend_from_slice(&[0x48, 0x81, 0xec, 0xa8, 0x00, 0x00, 0x00]); // sub rsp, 0xa8
    code.extend_from_slice(&[0xf3, 0x0f, 0x7f, 0x4c, 0x24, 0x20]); // movdqu [rsp+20], xmm1
    code.extend_from_slice(&[0x48, 0x89, 0x44, 0x24, 0x30]); // mov [rsp+30], rax
    code.extend_from_slice(&[0x48, 0x89, 0x4c, 0x24, 0x38]); // mov [rsp+38], rcx
    code.extend_from_slice(&[0x48, 0x89, 0x54, 0x24, 0x40]); // mov [rsp+40], rdx
    code.extend_from_slice(&[0x4c, 0x89, 0x44, 0x24, 0x48]); // mov [rsp+48], r8
    code.extend_from_slice(&[0x4c, 0x89, 0x4c, 0x24, 0x50]); // mov [rsp+50], r9
    code.extend_from_slice(&[0x4c, 0x89, 0x54, 0x24, 0x58]); // mov [rsp+58], r10
    code.extend_from_slice(&[0x4c, 0x89, 0x5c, 0x24, 0x60]); // mov [rsp+60], r11
    push_mov_rax_call(&mut code, before_hook);
    code.extend_from_slice(&[0x48, 0x8b, 0x44, 0x24, 0x30]); // mov rax, [rsp+30]
    code.extend_from_slice(&[0x48, 0x8b, 0x4c, 0x24, 0x38]); // mov rcx, [rsp+38]
    code.extend_from_slice(&[0x48, 0x8b, 0x54, 0x24, 0x40]); // mov rdx, [rsp+40]
    code.extend_from_slice(&[0x4c, 0x8b, 0x44, 0x24, 0x48]); // mov r8, [rsp+48]
    code.extend_from_slice(&[0x4c, 0x8b, 0x4c, 0x24, 0x50]); // mov r9, [rsp+50]
    code.extend_from_slice(&[0x4c, 0x8b, 0x54, 0x24, 0x58]); // mov r10, [rsp+58]
    code.extend_from_slice(&[0x4c, 0x8b, 0x5c, 0x24, 0x60]); // mov r11, [rsp+60]
    code.extend_from_slice(&[0xf3, 0x0f, 0x6f, 0x4c, 0x24, 0x20]); // movdqu xmm1, [rsp+20]
    code.extend_from_slice(&[0x48, 0x8b, 0x01]); // mov rax, [rcx]
    code.extend_from_slice(&[0xff, 0x50, 0x18]); // call qword ptr [rax+18]
    code.extend_from_slice(&[0x48, 0x89, 0x44, 0x24, 0x68]); // mov [rsp+68], rax
    push_mov_rax_call(&mut code, after_hook);
    code.extend_from_slice(&[0x48, 0x8b, 0x44, 0x24, 0x68]); // mov rax, [rsp+68]
    code.extend_from_slice(&[0x48, 0x81, 0xc4, 0xa8, 0x00, 0x00, 0x00]); // add rsp, 0xa8
    code.extend_from_slice(&[0x49, 0xbb]);
    code.extend_from_slice(&(resume as u64).to_le_bytes()); // mov r11, resume
    code.extend_from_slice(&[0x4c, 0x89, 0x1c, 0x24]); // mov [rsp], r11
    code.push(0xc3); // ret

    let bridge = win::allocate_executable_memory_near(anchor, code.len())?;
    std::ptr::copy_nonoverlapping(code.as_ptr(), bridge as *mut u8, code.len());
    let _ = win::flush_instruction_cache(bridge, code.len());
    Some(bridge)
}

unsafe fn create_costume_preview_resource_writer_block_bridge(
    anchor: usize,
    resume: usize,
    hook: usize,
) -> Option<usize> {
    let mut code = Vec::with_capacity(180);
    code.extend_from_slice(&[0x48, 0x81, 0xec, 0xa8, 0x00, 0x00, 0x00]); // sub rsp, 0xa8
    code.extend_from_slice(&[0x48, 0x89, 0x44, 0x24, 0x30]); // mov [rsp+30], rax
    code.extend_from_slice(&[0x48, 0x89, 0x4c, 0x24, 0x38]); // mov [rsp+38], rcx
    code.extend_from_slice(&[0x48, 0x89, 0x54, 0x24, 0x40]); // mov [rsp+40], rdx
    code.extend_from_slice(&[0x4c, 0x89, 0x44, 0x24, 0x48]); // mov [rsp+48], r8
    code.extend_from_slice(&[0x4c, 0x89, 0x4c, 0x24, 0x50]); // mov [rsp+50], r9
    code.extend_from_slice(&[0x4c, 0x89, 0x54, 0x24, 0x58]); // mov [rsp+58], r10
    code.extend_from_slice(&[0x4c, 0x89, 0x5c, 0x24, 0x60]); // mov [rsp+60], r11
    code.extend_from_slice(&[0x48, 0x89, 0x7c, 0x24, 0x68]); // mov [rsp+68], rdi
    code.extend_from_slice(&[0x48, 0x89, 0x5c, 0x24, 0x70]); // mov [rsp+70], rbx
    code.extend_from_slice(&[0x4c, 0x89, 0x64, 0x24, 0x78]); // mov [rsp+78], r12
    code.extend_from_slice(&[0x48, 0x89, 0xf9]); // mov rcx, rdi
    code.extend_from_slice(&[0x48, 0x89, 0xda]); // mov rdx, rbx
    code.extend_from_slice(&[0x4d, 0x89, 0xe0]); // mov r8, r12
    push_mov_rax_call(&mut code, hook);
    code.extend_from_slice(&[0x48, 0x8b, 0x44, 0x24, 0x30]); // mov rax, [rsp+30]
    code.extend_from_slice(&[0x48, 0x8b, 0x4c, 0x24, 0x38]); // mov rcx, [rsp+38]
    code.extend_from_slice(&[0x48, 0x8b, 0x54, 0x24, 0x40]); // mov rdx, [rsp+40]
    code.extend_from_slice(&[0x4c, 0x8b, 0x44, 0x24, 0x48]); // mov r8, [rsp+48]
    code.extend_from_slice(&[0x4c, 0x8b, 0x4c, 0x24, 0x50]); // mov r9, [rsp+50]
    code.extend_from_slice(&[0x4c, 0x8b, 0x54, 0x24, 0x58]); // mov r10, [rsp+58]
    code.extend_from_slice(&[0x4c, 0x8b, 0x5c, 0x24, 0x60]); // mov r11, [rsp+60]
    code.extend_from_slice(&[0x48, 0x8b, 0x7c, 0x24, 0x68]); // mov rdi, [rsp+68]
    code.extend_from_slice(&[0x48, 0x8b, 0x5c, 0x24, 0x70]); // mov rbx, [rsp+70]
    code.extend_from_slice(&[0x4c, 0x8b, 0x64, 0x24, 0x78]); // mov r12, [rsp+78]
    code.extend_from_slice(&COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_BYTES);
    code.extend_from_slice(&[0x48, 0x81, 0xc4, 0xa8, 0x00, 0x00, 0x00]); // add rsp, 0xa8
    code.extend_from_slice(&absolute_jump_bytes(resume));

    let bridge = win::allocate_executable_memory_near(anchor, code.len())?;
    std::ptr::copy_nonoverlapping(code.as_ptr(), bridge as *mut u8, code.len());
    let _ = win::flush_instruction_cache(bridge, code.len());
    Some(bridge)
}

unsafe fn create_costume_preview_resource_writer_block_snapshot_bridge(
    anchor: usize,
    resume: usize,
) -> Option<usize> {
    let mut code = Vec::with_capacity(220);
    code.push(0x9c); // pushfq
    code.push(0x50); // push rax
    code.extend_from_slice(&[0x41, 0x52]); // push r10
    code.extend_from_slice(&[0x41, 0x53]); // push r11
    code.extend_from_slice(&[0x48, 0xb8]);
    code.extend_from_slice(
        &(RESOURCE_911_WRITER_BLOCK_SNAPSHOT_SEQ.as_ptr() as usize as u64).to_le_bytes(),
    );
    code.extend_from_slice(&[0x49, 0xba]);
    code.extend_from_slice(&1u64.to_le_bytes()); // mov r10, 1
    code.extend_from_slice(&[0xf0, 0x4c, 0x0f, 0xc1, 0x10]); // lock xadd [rax], r10
    code.extend_from_slice(&[0x49, 0xff, 0xc2]); // inc r10
    code.extend_from_slice(&[0x4c, 0x89, 0xd0]); // mov rax, r10
    code.extend_from_slice(&[0x48, 0x83, 0xe0, RESOURCE_911_WRITER_BLOCK_RING_MASK as u8]); // and rax, ring mask
    code.extend_from_slice(&[0x48, 0xc1, 0xe0, 0x03]); // shl rax, 3
    push_mov_r11_store_indexed_reg(
        &mut code,
        RESOURCE_911_WRITER_BLOCK_SNAPSHOT_SEQ_RING[0].as_ptr() as usize,
        &[0x4d, 0x89, 0x14, 0x03], // mov [r11+rax], r10
    );
    push_mov_r11_store_indexed_reg(
        &mut code,
        RESOURCE_911_WRITER_BLOCK_SNAPSHOT_RDI_RING[0].as_ptr() as usize,
        &[0x49, 0x89, 0x3c, 0x03], // mov [r11+rax], rdi
    );
    push_mov_r11_store_indexed_reg(
        &mut code,
        RESOURCE_911_WRITER_BLOCK_SNAPSHOT_RBX_RING[0].as_ptr() as usize,
        &[0x49, 0x89, 0x1c, 0x03], // mov [r11+rax], rbx
    );
    push_mov_r11_store_indexed_reg(
        &mut code,
        RESOURCE_911_WRITER_BLOCK_SNAPSHOT_R12_RING[0].as_ptr() as usize,
        &[0x4d, 0x89, 0x24, 0x03], // mov [r11+rax], r12
    );
    code.extend_from_slice(&[0x41, 0x5b]); // pop r11
    code.extend_from_slice(&[0x41, 0x5a]); // pop r10
    code.push(0x58); // pop rax
    code.push(0x9d); // popfq
    code.extend_from_slice(&COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_BYTES);
    code.extend_from_slice(&absolute_jump_bytes(resume));

    let bridge = win::allocate_executable_memory_near(anchor, code.len())?;
    std::ptr::copy_nonoverlapping(code.as_ptr(), bridge as *mut u8, code.len());
    let _ = win::flush_instruction_cache(bridge, code.len());
    Some(bridge)
}

fn push_mov_r11_store_indexed_reg(
    code: &mut Vec<u8>,
    destination: usize,
    store_instruction: &[u8],
) {
    code.extend_from_slice(&[0x49, 0xbb]);
    code.extend_from_slice(&(destination as u64).to_le_bytes());
    code.extend_from_slice(store_instruction);
}

fn push_mov_rax_call(code: &mut Vec<u8>, destination: usize) {
    code.extend_from_slice(&[0x48, 0xb8]);
    code.extend_from_slice(&(destination as u64).to_le_bytes());
    code.extend_from_slice(&[0xff, 0xd0]);
}

fn relative_call_bytes(callsite: usize, destination: usize) -> Option<[u8; 5]> {
    let next_instruction = callsite.checked_add(COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_LEN)?;
    let delta = (destination as i128) - (next_instruction as i128);
    let offset = i32::try_from(delta).ok()?;
    let mut bytes = [0u8; COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_LEN];
    bytes[0] = 0xe8;
    bytes[1..].copy_from_slice(&offset.to_le_bytes());
    Some(bytes)
}

fn relative_jump_bytes(callsite: usize, destination: usize) -> Option<[u8; 5]> {
    let next_instruction = callsite.checked_add(COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_LEN)?;
    let delta = (destination as i128) - (next_instruction as i128);
    let offset = i32::try_from(delta).ok()?;
    let mut bytes = [0u8; COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_LEN];
    bytes[0] = 0xe9;
    bytes[1..].copy_from_slice(&offset.to_le_bytes());
    Some(bytes)
}

fn patch_exact_bytes(address: usize, patch: &[u8]) -> bool {
    let mut old_protect = 0;
    if !unsafe { win::make_memory_writable(address as Lpvoid, patch.len(), &mut old_protect) } {
        return false;
    }
    let written = win::write_process_memory(address, patch);
    let _ = unsafe { win::restore_memory_protection(address as Lpvoid, patch.len(), old_protect) };
    if !matches!(written, Some(count) if count == patch.len()) {
        return false;
    }
    win::flush_instruction_cache(address, patch.len())
}

unsafe fn create_ui_child_toggle_trampoline(target: usize) -> Option<usize> {
    const PREFIX_LEN: usize = 8;
    const SHORT_JE_OFFSET: usize = 8;
    const SHORT_JE_LEN: usize = 2;
    const COPIED_AFTER_BRANCH_OFFSET: usize = PREFIX_LEN + SHORT_JE_LEN;
    const ORIGINAL_BRANCH_TARGET_OFFSET: usize = 0x70;
    const JNE_OVER_ABSOLUTE_JUMP: [u8; 2] = [0x75, ABSOLUTE_JUMP_LEN as u8];
    const EXPECTED_PREFIX: [u8; UI_CHILD_TOGGLE_STOLEN_LEN] = [
        0x48, 0x83, 0x79, 0x28, 0x00, 0x45, 0x8b, 0xc8, 0x74, 0x66, 0x48, 0x63, 0x81, 0xb4, 0x00,
        0x00, 0x00,
    ];

    let mut original = [0u8; UI_CHILD_TOGGLE_STOLEN_LEN];
    if !read_exact_process_memory(target, &mut original) {
        return None;
    }
    if original != EXPECTED_PREFIX {
        log::write_line(format!(
            "ui child toggle trampoline skipped target=0x{target:x} error=unexpected_prologue bytes={}",
            format_bytes(&original)
        ));
        return None;
    }

    let copied_tail_len = UI_CHILD_TOGGLE_STOLEN_LEN - COPIED_AFTER_BRANCH_OFFSET;
    let trampoline_size = PREFIX_LEN
        + JNE_OVER_ABSOLUTE_JUMP.len()
        + ABSOLUTE_JUMP_LEN
        + copied_tail_len
        + ABSOLUTE_JUMP_LEN;
    let trampoline = win::allocate_executable_memory(trampoline_size)?;
    std::ptr::copy_nonoverlapping(original.as_ptr(), trampoline as *mut u8, PREFIX_LEN);
    std::ptr::copy_nonoverlapping(
        JNE_OVER_ABSOLUTE_JUMP.as_ptr(),
        (trampoline + SHORT_JE_OFFSET) as *mut u8,
        SHORT_JE_LEN,
    );

    let branch_jump = absolute_jump_bytes(target + ORIGINAL_BRANCH_TARGET_OFFSET);
    std::ptr::copy_nonoverlapping(
        branch_jump.as_ptr(),
        (trampoline + PREFIX_LEN + SHORT_JE_LEN) as *mut u8,
        branch_jump.len(),
    );

    let copied_tail_destination = trampoline + PREFIX_LEN + SHORT_JE_LEN + ABSOLUTE_JUMP_LEN;
    std::ptr::copy_nonoverlapping(
        original[COPIED_AFTER_BRANCH_OFFSET..].as_ptr(),
        copied_tail_destination as *mut u8,
        copied_tail_len,
    );

    let jump_back = absolute_jump_bytes(target + UI_CHILD_TOGGLE_STOLEN_LEN);
    std::ptr::copy_nonoverlapping(
        jump_back.as_ptr(),
        (copied_tail_destination + copied_tail_len) as *mut u8,
        jump_back.len(),
    );
    let _ = win::flush_instruction_cache(trampoline, trampoline_size);
    Some(trampoline)
}

unsafe fn create_costume_companion_preview_update_trampoline(
    module: usize,
    target: usize,
) -> Option<usize> {
    const PLAIN_PREFIX_LEN: usize = 12;
    const MOV_RAX_IMM64_LOAD_RAX_BYTES: usize = 13;
    let mut original = [0u8; COSTUME_COMPANION_PREVIEW_UPDATE_STOLEN_LEN];
    if !read_exact_process_memory(target, &mut original) {
        return None;
    }

    let expected = [
        0x40, 0x55, 0x57, 0x41, 0x56, 0x48, 0x81, 0xec, 0xb0, 0x00, 0x00, 0x00, 0x48, 0x8b, 0x05,
        0xcd, 0x73, 0x92, 0x00,
    ];
    if original != expected {
        log::write_line(format!(
            "costume companion preview-update trampoline skipped target=0x{target:x} error=unexpected_prologue bytes={}",
            format_bytes(&original)
        ));
        return None;
    }

    let trampoline_size = PLAIN_PREFIX_LEN + MOV_RAX_IMM64_LOAD_RAX_BYTES + ABSOLUTE_JUMP_LEN;
    let trampoline = win::allocate_executable_memory(trampoline_size)?;
    std::ptr::copy_nonoverlapping(original.as_ptr(), trampoline as *mut u8, PLAIN_PREFIX_LEN);

    let data_address = module + COSTUME_PREVIEW_VISIBLE_BRANCH_COOKIE_GLOBAL_RVA;
    let mut relocated_load = Vec::with_capacity(MOV_RAX_IMM64_LOAD_RAX_BYTES);
    relocated_load.extend_from_slice(&[0x48, 0xb8]);
    relocated_load.extend_from_slice(&(data_address as u64).to_le_bytes());
    relocated_load.extend_from_slice(&[0x48, 0x8b, 0x00]);
    std::ptr::copy_nonoverlapping(
        relocated_load.as_ptr(),
        (trampoline + PLAIN_PREFIX_LEN) as *mut u8,
        relocated_load.len(),
    );

    let jump_back = absolute_jump_bytes(target + COSTUME_COMPANION_PREVIEW_UPDATE_STOLEN_LEN);
    std::ptr::copy_nonoverlapping(
        jump_back.as_ptr(),
        (trampoline + PLAIN_PREFIX_LEN + relocated_load.len()) as *mut u8,
        jump_back.len(),
    );
    let _ = win::flush_instruction_cache(trampoline, trampoline_size);
    Some(trampoline)
}

unsafe fn create_costume_preview_visible_branch_trampoline(
    module: usize,
    target: usize,
) -> Option<usize> {
    const PLAIN_PREFIX_LEN: usize = 10;
    const MOV_RAX_IMM64_LOAD_RAX_BYTES: usize = 13;
    let mut original = [0u8; COSTUME_PREVIEW_VISIBLE_BRANCH_STOLEN_LEN];
    if !read_exact_process_memory(target, &mut original) {
        return None;
    }

    let expected = [
        0x40, 0x53, 0x57, 0x48, 0x81, 0xec, 0x38, 0x08, 0x00, 0x00, 0x48, 0x8b, 0x05, 0x8f, 0x5b,
        0x92, 0x00,
    ];
    if original != expected {
        log::write_line(format!(
            "costume preview visible-branch trampoline skipped target=0x{target:x} error=unexpected_prologue bytes={}",
            format_bytes(&original)
        ));
        return None;
    }

    let trampoline_size = PLAIN_PREFIX_LEN + MOV_RAX_IMM64_LOAD_RAX_BYTES + ABSOLUTE_JUMP_LEN;
    let trampoline = win::allocate_executable_memory(trampoline_size)?;
    std::ptr::copy_nonoverlapping(original.as_ptr(), trampoline as *mut u8, PLAIN_PREFIX_LEN);

    let data_address = module + COSTUME_PREVIEW_VISIBLE_BRANCH_COOKIE_GLOBAL_RVA;
    let mut relocated_load = Vec::with_capacity(MOV_RAX_IMM64_LOAD_RAX_BYTES);
    relocated_load.extend_from_slice(&[0x48, 0xb8]);
    relocated_load.extend_from_slice(&(data_address as u64).to_le_bytes());
    relocated_load.extend_from_slice(&[0x48, 0x8b, 0x00]);
    std::ptr::copy_nonoverlapping(
        relocated_load.as_ptr(),
        (trampoline + PLAIN_PREFIX_LEN) as *mut u8,
        relocated_load.len(),
    );

    let jump_back = absolute_jump_bytes(target + COSTUME_PREVIEW_VISIBLE_BRANCH_STOLEN_LEN);
    std::ptr::copy_nonoverlapping(
        jump_back.as_ptr(),
        (trampoline + PLAIN_PREFIX_LEN + relocated_load.len()) as *mut u8,
        jump_back.len(),
    );
    let _ = win::flush_instruction_cache(trampoline, trampoline_size);
    Some(trampoline)
}

unsafe fn patch_absolute_jump(target: usize, replacement: usize, stolen_len: usize) -> bool {
    if stolen_len < ABSOLUTE_JUMP_LEN {
        return false;
    }

    let mut patch = vec![0x90; stolen_len];
    let jump = absolute_jump_bytes(replacement);
    patch[..jump.len()].copy_from_slice(&jump);

    let mut old_protect = 0;
    if !win::make_memory_writable(target as Lpvoid, patch.len(), &mut old_protect) {
        return false;
    }
    let written = win::write_process_memory(target, &patch);
    let _ = win::restore_memory_protection(target as Lpvoid, patch.len(), old_protect);
    if !matches!(written, Some(count) if count == patch.len()) {
        return false;
    }
    win::flush_instruction_cache(target, patch.len())
}

fn patch_single_code_byte(address: usize, expected: u8, target: u8, label: &str) -> bool {
    let mut before_bytes = [0u8; 1];
    if !read_exact_process_memory(address, &mut before_bytes) {
        log::write_line(format!(
            "{label} patch failed address=0x{address:x} error=read_before_failed"
        ));
        return false;
    }

    let before = before_bytes[0];
    if before == target {
        log::write_line(format!(
            "{label} patch already present address=0x{address:x} value=0x{before:02x}"
        ));
        return true;
    }
    if before != expected {
        log::write_line(format!(
            "{label} patch skipped address=0x{address:x} before=0x{before:02x} expected=0x{expected:02x} target=0x{target:02x}"
        ));
        return false;
    }

    let mut old_protect = 0;
    if !unsafe { win::make_memory_writable(address as Lpvoid, 1, &mut old_protect) } {
        log::write_line(format!(
            "{label} patch failed address=0x{address:x} before=0x{before:02x} target=0x{target:02x} error=protect_failed"
        ));
        return false;
    }

    let patch = [target];
    let written = win::write_process_memory(address, &patch);
    let _ = unsafe { win::restore_memory_protection(address as Lpvoid, 1, old_protect) };
    let flushed = win::flush_instruction_cache(address, 1);
    if !matches!(written, Some(count) if count == patch.len()) || !flushed {
        log::write_line(format!(
            "{label} patch failed address=0x{address:x} before=0x{before:02x} target=0x{target:02x} error=write_failed"
        ));
        return false;
    }

    let mut after_bytes = [0u8; 1];
    let after = if read_exact_process_memory(address, &mut after_bytes) {
        after_bytes[0]
    } else {
        0
    };
    log::write_line(format!(
        "{label} patch address=0x{address:x} before=0x{before:02x} after=0x{after:02x} target=0x{target:02x}"
    ));
    after == target
}

fn absolute_jump_bytes(target: usize) -> [u8; ABSOLUTE_JUMP_LEN] {
    let mut bytes = [0u8; ABSOLUTE_JUMP_LEN];
    bytes[..6].copy_from_slice(&[0xff, 0x25, 0x00, 0x00, 0x00, 0x00]);
    bytes[6..].copy_from_slice(&(target as u64).to_le_bytes());
    bytes
}

unsafe fn import_directory(module: usize) -> Option<(u32, u32)> {
    let dos = module as *const u8;
    if std::slice::from_raw_parts(dos, 2) != b"MZ" {
        return None;
    }
    let nt_offset = *(dos.add(0x3c) as *const u32) as usize;
    let nt = dos.add(nt_offset);
    if std::slice::from_raw_parts(nt, 4) != b"PE\0\0" {
        return None;
    }

    let optional_header = nt.add(24);
    let magic = *(optional_header as *const u16);
    if magic != 0x20b {
        return None;
    }

    let data_directories = optional_header.add(112);
    let import_directory = data_directories.add(IMAGE_DIRECTORY_ENTRY_IMPORT * 8);
    let rva = *(import_directory as *const u32);
    let size = *(import_directory.add(4) as *const u32);
    Some((rva, size))
}

unsafe fn patch_import_descriptor(
    module: usize,
    descriptor: &ImageImportDescriptor,
    originals: &mut OriginalFunctions,
) {
    let lookup_rva = if descriptor.original_first_thunk != 0 {
        descriptor.original_first_thunk
    } else {
        descriptor.first_thunk
    };
    if lookup_rva == 0 || descriptor.first_thunk == 0 {
        return;
    }

    let mut lookup = (module + lookup_rva as usize) as *const u64;
    let mut iat = (module + descriptor.first_thunk as usize) as *mut usize;
    while *lookup != 0 {
        let entry = *lookup;
        if entry & IMAGE_ORDINAL_FLAG64 == 0 {
            let import_by_name = (module + entry as usize) as *const u8;
            let name = c_string_at(import_by_name.add(2));
            patch_import_by_name(name, iat, originals);
        }
        lookup = lookup.add(1);
        iat = iat.add(1);
    }
}

unsafe fn patch_import_by_name(name: &str, iat: *mut usize, originals: &mut OriginalFunctions) {
    if steam_probe_enabled() {
        if let Some(version) = steam_apps_import_version(name) {
            patch_steam_apps_interface_import(version, iat, originals);
            return;
        }
        if let Some(method) = steam_apps_bool_method_import(name) {
            patch_steam_apps_bool_method_import(method, iat, originals);
            return;
        }
    }

    match name {
        "CreateFileW" => patch_slot(iat, hooked_create_file_w as usize, |original| {
            originals.create_file_w = Some(std::mem::transmute(original));
        }),
        "ReadFile" => patch_slot(iat, hooked_read_file as usize, |original| {
            originals.read_file = Some(std::mem::transmute(original));
        }),
        "CloseHandle" => patch_slot(iat, hooked_close_handle as usize, |original| {
            originals.close_handle = Some(std::mem::transmute(original));
        }),
        "GetFileSizeEx" => patch_slot(iat, hooked_get_file_size_ex as usize, |original| {
            originals.get_file_size_ex = Some(std::mem::transmute(original));
        }),
        "GetFileTime" => patch_slot(iat, hooked_get_file_time as usize, |original| {
            originals.get_file_time = Some(std::mem::transmute(original));
        }),
        "GetFileType" => patch_slot(iat, hooked_get_file_type as usize, |original| {
            originals.get_file_type = Some(std::mem::transmute(original));
        }),
        "SetFilePointerEx" => patch_slot(iat, hooked_set_file_pointer_ex as usize, |original| {
            originals.set_file_pointer_ex = Some(std::mem::transmute(original));
        }),
        "GetFileAttributesW" => {
            patch_slot(iat, hooked_get_file_attributes_w as usize, |original| {
                originals.get_file_attributes_w = Some(std::mem::transmute(original));
            })
        }
        "GetFileAttributesExW" => {
            patch_slot(iat, hooked_get_file_attributes_ex_w as usize, |original| {
                originals.get_file_attributes_ex_w = Some(std::mem::transmute(original));
            })
        }
        "FindFirstFileW" => patch_slot(iat, hooked_find_first_file_w as usize, |original| {
            originals.find_first_file_w = Some(std::mem::transmute(original));
        }),
        "FindFirstFileExW" => patch_slot(iat, hooked_find_first_file_ex_w as usize, |original| {
            originals.find_first_file_ex_w = Some(std::mem::transmute(original));
        }),
        "FindNextFileW" => patch_slot(iat, hooked_find_next_file_w as usize, |original| {
            originals.find_next_file_w = Some(std::mem::transmute(original));
        }),
        "FindClose" => patch_slot(iat, hooked_find_close as usize, |original| {
            originals.find_close = Some(std::mem::transmute(original));
        }),
        "GetProcAddress" if steam_probe_enabled() => {
            patch_slot(iat, hooked_get_proc_address as usize, |original| {
                originals.get_proc_address = Some(std::mem::transmute(original));
            })
        }
        _ => {}
    }
}

fn steam_probe_enabled() -> bool {
    STEAM_PROBE_ENABLED.load(Ordering::Relaxed)
}

unsafe fn patch_steam_apps_interface_import(
    version: &str,
    iat: *mut usize,
    originals: &mut OriginalFunctions,
) {
    match version {
        "v006" => patch_slot(iat, hooked_steam_api_steam_apps_v006 as usize, |original| {
            originals.steam_apps_v006 = Some(std::mem::transmute(original));
        }),
        "v007" => patch_slot(iat, hooked_steam_api_steam_apps_v007 as usize, |original| {
            originals.steam_apps_v007 = Some(std::mem::transmute(original));
        }),
        "v008" => patch_slot(iat, hooked_steam_api_steam_apps_v008 as usize, |original| {
            originals.steam_apps_v008 = Some(std::mem::transmute(original));
        }),
        _ => {}
    }
}

unsafe fn patch_steam_apps_bool_method_import(
    method: SteamAppsBoolMethod,
    iat: *mut usize,
    originals: &mut OriginalFunctions,
) {
    match method {
        SteamAppsBoolMethod::IsSubscribedApp => patch_slot(
            iat,
            hooked_steam_apps_flat_b_is_subscribed_app as usize,
            |original| {
                originals.steam_apps_b_is_subscribed_app = Some(std::mem::transmute(original));
            },
        ),
        SteamAppsBoolMethod::IsDlcInstalled => patch_slot(
            iat,
            hooked_steam_apps_flat_b_is_dlc_installed as usize,
            |original| {
                originals.steam_apps_b_is_dlc_installed = Some(std::mem::transmute(original));
            },
        ),
    }
}

unsafe fn patch_slot<F>(slot: *mut usize, replacement: usize, assign_original: F)
where
    F: FnOnce(usize),
{
    if *slot == replacement {
        return;
    }
    let original = *slot;
    let mut old_protect = 0;
    if !win::make_memory_writable(slot.cast(), size_of::<usize>(), &mut old_protect) {
        return;
    }
    *slot = replacement;
    let _ = win::restore_memory_protection(slot.cast(), size_of::<usize>(), old_protect);
    assign_original(original);
}

unsafe fn c_string_at(ptr: *const u8) -> &'static str {
    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    let bytes = std::slice::from_raw_parts(ptr, len);
    std::str::from_utf8_unchecked(bytes)
}

fn count_installed(originals: OriginalFunctions) -> usize {
    [
        originals.create_file_w.is_some(),
        originals.read_file.is_some(),
        originals.close_handle.is_some(),
        originals.get_file_size_ex.is_some(),
        originals.get_file_time.is_some(),
        originals.get_file_type.is_some(),
        originals.set_file_pointer_ex.is_some(),
        originals.get_file_attributes_w.is_some(),
        originals.get_file_attributes_ex_w.is_some(),
        originals.find_first_file_w.is_some(),
        originals.find_first_file_ex_w.is_some(),
        originals.find_next_file_w.is_some(),
        originals.find_close.is_some(),
        originals.get_proc_address.is_some(),
        originals.steam_apps_v006.is_some(),
        originals.steam_apps_v007.is_some(),
        originals.steam_apps_v008.is_some(),
        originals.steam_apps_b_is_subscribed_app.is_some(),
        originals.steam_apps_b_is_dlc_installed.is_some(),
    ]
    .into_iter()
    .filter(|installed| *installed)
    .count()
}

include!("costume.rs");
include!("model.rs");
include!("io.rs");
include!("core.rs");
include!("legacy/negative_probes.rs");
#[cfg(test)]
mod tests {
    use super::*;

    static HOOKS_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct AtomicUsizeRestore {
        target: &'static AtomicUsize,
        previous: usize,
    }

    impl Drop for AtomicUsizeRestore {
        fn drop(&mut self) {
            self.target.store(self.previous, Ordering::Relaxed);
        }
    }

    fn set_atomic_usize_for_test(target: &'static AtomicUsize, value: usize) -> AtomicUsizeRestore {
        let previous = target.swap(value, Ordering::Relaxed);
        AtomicUsizeRestore { target, previous }
    }

    struct AtomicBoolRestore {
        target: &'static AtomicBool,
        previous: bool,
    }

    impl Drop for AtomicBoolRestore {
        fn drop(&mut self) {
            self.target.store(self.previous, Ordering::Relaxed);
        }
    }

    fn set_atomic_bool_for_test(target: &'static AtomicBool, value: bool) -> AtomicBoolRestore {
        let previous = target.swap(value, Ordering::Relaxed);
        AtomicBoolRestore { target, previous }
    }

    #[test]
    fn fake_handle_round_trip_preserves_id() {
        let handle = VirtualHandle::from_raw(42);

        let round_trip = fake_to_handle(handle_to_fake(handle));

        assert_eq!(round_trip.map(VirtualHandle::as_raw), Some(42));
    }

    #[test]
    fn normal_handles_are_not_virtual() {
        assert_eq!(fake_to_handle(std::ptr::null_mut()), None);
        assert_eq!(fake_to_handle(INVALID_HANDLE_VALUE), None);
    }

    #[test]
    fn virtual_open_returns_fake_handle_value() {
        let virtual_handle = VirtualHandle::from_raw(0x21);

        let returned = returned_virtual_handle(virtual_handle);

        assert_eq!(
            fake_to_handle(returned).map(VirtualHandle::as_raw),
            Some(0x21)
        );
    }

    #[test]
    fn fake_handle_preserves_runtime_kind() {
        let handle = VirtualHandle::from_raw(0x42);
        let fake = handle_to_fake_for_runtime(handle, VirtualRuntimeKind::LawCustomSlot);

        let decoded = fake_virtual_handle(fake);

        assert_eq!(decoded, Some((handle, VirtualRuntimeKind::LawCustomSlot)));
        assert_eq!(fake_to_handle(fake), Some(handle));
    }

    #[test]
    fn law_custom_slot_private_model_diagnostic_targets_cloned_law_resource_row() {
        assert!(!law_custom_slot_index_patch_enabled());
        assert!(!LAW_CUSTOM_SLOT_DORMANT_ASSETS_ALWAYS_ACTIVE);
        assert_eq!(
            law_custom_slot_open_runtime_kind(false),
            VirtualRuntimeKind::LawCustomOriginal
        );
        assert_eq!(
            law_custom_slot_open_runtime_kind(true),
            VirtualRuntimeKind::LawCustomSlot
        );
        assert!(LAW_EXTRA_SLOT_CUSTOM_MODEL_RESOURCE_PATCH_ENABLED);
        assert!(!LAW_EXTRA_SLOT_PRIVATE_MODEL_RAM_CLONE_ENABLED);
        assert_eq!(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID, 26);
        assert_eq!(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID, 292);
        assert_eq!(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_NAME, "MPLC026_Law");
        assert_eq!(law_extra_slot_model_resource_for_patch(true, false), 292);
    }

    #[test]
    fn law_extra_slot_model_resource_stays_private_when_manager_alias_is_enabled() {
        assert!(!LAW_EXTRA_SLOT_PRIVATE_MODEL_MANAGER_ALIAS_ENABLED);
        assert_eq!(law_extra_slot_model_resource_for_patch(false, true), 292);
        assert_eq!(law_extra_slot_model_resource_for_patch(false, false), 292);
    }

    #[test]
    fn law_private_model_manager_alias_maps_only_private_probe_id() {
        assert_eq!(
            law_private_model_manager_resource_id(u32::from(
                LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID
            )),
            u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        );
        assert_eq!(
            law_private_model_manager_resource_id(u32::from(
                LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID
            )),
            u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)
        );
        assert_eq!(law_private_model_manager_resource_id(272), 272);
    }

    #[test]
    fn law_private_model_can_start_diagnostic_only_unblocks_initial_private_mirror() {
        let initial_private_mirror = ModelResourceSlotMirror {
            source_pointer: 0,
            source_state: 1,
            before_pointer: 0,
            before_state: 0,
            after_pointer: 0,
            after_state: 1,
            write_attempted: true,
            write_ok: true,
            after_matches: true,
        };

        assert!(!LAW_EXTRA_SLOT_PRIVATE_MODEL_CAN_START_DIAGNOSTIC_ENABLED);
        assert_eq!(
            law_private_model_can_start_effective_result(
                u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID),
                Some(initial_private_mirror),
                0
            ),
            (0, false)
        );
        assert_eq!(
            law_private_model_can_start_effective_result(
                u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID),
                Some(ModelResourceSlotMirror {
                    before_state: 1,
                    ..initial_private_mirror
                }),
                0
            ),
            (0, false)
        );
        assert_eq!(
            law_private_model_can_start_effective_result(
                u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID),
                u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID),
                Some(initial_private_mirror),
                0
            ),
            (0, false)
        );
    }

    #[test]
    fn model_resource_manager_entry_address_uses_game_stride() {
        let manager = 0x1000;

        assert_eq!(
            model_resource_manager_entry_address(manager, 26),
            Some(
                manager
                    + MODEL_RESOURCE_MANAGER_ENTRY_OFFSET
                    + 26 * MODEL_RESOURCE_MANAGER_ENTRY_STRIDE
            )
        );
        assert_eq!(
            MODEL_RESOURCE_MANAGER_ENTRY_STATE_OFFSET,
            MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET + size_of::<usize>()
        );
        assert_eq!(MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE, 0x20);
    }

    #[test]
    fn model_render_attach_probe_targets_ghidra_confirmed_function() {
        assert_eq!(MODEL_RENDER_ATTACH_RVA, 0x03ce790);
        assert_eq!(MODEL_RENDER_ATTACH_STOLEN_LEN, 15);
    }

    #[test]
    fn preview_resource_writer_block_hook_targets_confirmed_state_marker_writes() {
        assert_eq!(COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RVA, 0x1582de6);
        assert_eq!(COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_RESUME_RVA, 0x1582dfb);
        assert_eq!(
            COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_STOLEN_LEN,
            COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_BYTES.len()
        );
        assert_eq!(
            &COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_BYTES[..4],
            &[0x48, 0x89, 0x5f, 0xf0]
        );
        assert_eq!(
            &COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_BYTES[4..11],
            &[0x44, 0x89, 0xa7, 0x70, 0x01, 0x00, 0x00]
        );
        assert_eq!(
            &COSTUME_PREVIEW_RESOURCE_WRITER_BLOCK_BYTES[11..],
            &[0xc7, 0x87, 0x7c, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn model_color_apply_probe_targets_ghidra_confirmed_function() {
        assert_eq!(MODEL_COLOR_APPLY_RVA, 0x1354510);
        assert_eq!(MODEL_COLOR_APPLY_STOLEN_LEN, 19);
        assert!(MODEL_COLOR_APPLY_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
        assert!(is_interesting_model_color_apply(Some(292), 0));
        assert!(is_interesting_model_color_apply(None, u32::from(u16::MAX)));
        assert!(!is_interesting_model_color_apply(Some(911), 0));
    }

    #[test]
    fn model_resource_status_check_targets_ghidra_confirmed_function() {
        assert_eq!(MODEL_RESOURCE_STATUS_MANAGER_RVA, 0x1eba7b0);
        assert_eq!(MODEL_RESOURCE_STATUS_CHECK_RVA, 0x016e250);
        assert_eq!(MODEL_RESOURCE_STATUS_CHECK_STOLEN_LEN, 15);
        assert!(MODEL_RESOURCE_STATUS_CHECK_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
        assert!(is_interesting_model_resource_status_check(356_022));
        assert!(is_interesting_model_resource_status_check(420_000));
        assert!(!is_interesting_model_resource_status_check(26));
    }

    #[test]
    fn model_ready_wait_check_targets_ghidra_confirmed_function() {
        assert_eq!(MODEL_READY_WAIT_CHECK_RVA, 0x135af30);
        assert_eq!(MODEL_READY_WAIT_CHECK_STOLEN_LEN, 15);
        assert!(MODEL_READY_WAIT_CHECK_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
    }

    #[test]
    fn model_load_state_step_targets_ghidra_confirmed_state_machine() {
        assert_eq!(MODEL_LOAD_STATE_STEP_RVA, 0x129e8c0);
        assert_eq!(MODEL_LOAD_STATE_STEP_STOLEN_LEN, 14);
        assert!(MODEL_LOAD_STATE_STEP_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
        assert!(is_interesting_model_ready_resource(308));
        assert!(is_interesting_model_ready_resource(292));
        assert!(!is_interesting_model_ready_resource(911));
    }

    #[test]
    fn preview_model_branch_hooks_target_visible_setup_and_tail_update() {
        assert_eq!(COSTUME_PREVIEW_VISIBLE_BRANCH_RVA, 0x148af40);
        assert_eq!(COSTUME_PREVIEW_VISIBLE_BRANCH_STOLEN_LEN, 17);
        assert_eq!(COSTUME_PREVIEW_HIDDEN_BRANCH_RVA, 0x148ae20);
        assert_eq!(COSTUME_PREVIEW_HIDDEN_BRANCH_STOLEN_LEN, 16);
        assert_eq!(COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_RVA, 0x148ac40);
        assert_eq!(COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_STOLEN_LEN, 17);
        assert_eq!(COSTUME_COMPANION_PREVIEW_UPDATE_RVA, 0x1489700);
        assert_eq!(COSTUME_COMPANION_PREVIEW_UPDATE_STOLEN_LEN, 19);
        assert_eq!(COSTUME_PREVIEW_GLOBAL_DLC_GATE_RVA, 0x12ffbd0);
        assert_eq!(COSTUME_PREVIEW_GLOBAL_DLC_GATE_STOLEN_LEN, 14);
        assert_eq!(COSTUME_PREVIEW_LAYOUT_MODE_GATE_RVA, 0x12f9320);
        assert_eq!(COSTUME_PREVIEW_LAYOUT_MODE_GATE_STOLEN_LEN, 15);
        assert_eq!(COSTUME_PREVIEW_TAIL_UPDATE_RVA, 0x148bcc0);
        assert_eq!(COSTUME_PREVIEW_TAIL_UPDATE_STOLEN_LEN, 15);
        assert_eq!(COSTUME_PREVIEW_RESOURCE_ATTACH_RVA, 0x1613030);
        assert_eq!(COSTUME_PREVIEW_RESOURCE_ATTACH_STOLEN_LEN, 14);
        assert_eq!(COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_RVA, 0x1582c30);
        assert_eq!(COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_STOLEN_LEN, 15);
        assert_eq!(
            COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_BYTES,
            [
                0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
                0x18,
            ]
        );
        assert_eq!(COSTUME_PREVIEW_RESOURCE_RESOLVE_RVA, 0x1582f50);
        assert_eq!(COSTUME_PREVIEW_RESOURCE_RESOLVE_STOLEN_LEN, 17);
    }

    #[test]
    fn ui_child_toggle_hook_targets_apply_ready_child_helper() {
        assert_eq!(UI_CHILD_TOGGLE_RVA, 0x1604620);
        assert_eq!(UI_CHILD_TOGGLE_STOLEN_LEN, 17);
        assert!(UI_CHILD_TOGGLE_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
        assert!(!is_interesting_ui_child_toggle(0x1000, 0x36));
    }

    #[test]
    fn companion_preview_trace_filter_keeps_law_and_custom_calls() {
        let _lock = HOOKS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let _custom_inactive = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, false);

        assert!(is_interesting_companion_preview_update(0x1000, 26, 1, 1));
        assert!(!is_interesting_companion_preview_update(0, 26, 1, 1));
        assert!(!is_interesting_companion_preview_update(0x1000, 60, 1, 1));

        let _custom_active = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, true);
        assert!(is_interesting_companion_preview_update(0x1000, 60, 1, 1));
    }

    #[test]
    fn preview_model_branch_trace_filter_keeps_law_and_custom_calls() {
        let _lock = HOOKS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let _custom_inactive = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, false);
        let _probe_variant = set_atomic_usize_for_test(&LAW_EXTRA_SLOT_PROBE_VARIANT_ID, 699);

        assert!(preview_model_branch_trace_candidate(Some(26), None));
        assert!(preview_model_branch_trace_candidate(None, Some(699)));
        assert!(!preview_model_branch_trace_candidate(Some(60), Some(60)));

        let _custom_active = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, true);
        assert!(preview_model_branch_trace_candidate(Some(60), Some(60)));
    }

    #[test]
    fn preview_resource_attach_trace_filter_keeps_law_mapped_resources() {
        let _lock = HOOKS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let _custom_inactive = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, false);

        assert!(preview_resource_attach_trace_candidate(
            LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID
        ));
        assert!(preview_resource_attach_trace_candidate(
            LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
        ));
        assert!(!preview_resource_attach_trace_candidate(42));

        let _custom_active = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, true);
        assert!(preview_resource_attach_trace_candidate(42));
    }

    #[test]
    fn preview_resource_resolve_trace_filter_keeps_neighbor_diagnostic_resource() {
        let _lock = HOOKS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let _custom_inactive = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, false);

        assert!(preview_resource_resolve_trace_candidate(
            LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID
        ));
        assert!(preview_resource_resolve_trace_candidate(
            LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
        ));
        assert!(preview_resource_resolve_trace_candidate(1957));
        assert!(!preview_resource_resolve_trace_candidate(42));
    }

    #[test]
    fn preview_gate_trace_filter_keeps_layout_mode_calls_for_law_and_custom() {
        let _lock = HOOKS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let _custom_inactive = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, false);
        let _probe_variant = set_atomic_usize_for_test(&LAW_EXTRA_SLOT_PROBE_VARIANT_ID, 699);

        assert!(preview_gate_trace_interesting(26, 4, 1));
        assert!(preview_gate_trace_interesting(699, 0, 1));
        assert!(preview_gate_trace_interesting(60, 0, 0));
        assert!(!preview_gate_trace_interesting(60, 0, 1));

        let _custom_active = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, true);
        assert!(preview_gate_trace_interesting(60, 0, 1));
    }

    #[test]
    fn preview_model_update_gate_probe_rejects_out_of_range_layouts() {
        assert_eq!(preview_model_update_layout_preview_value(-1), None);
        assert_eq!(
            preview_model_update_layout_preview_value(COSTUME_LAYOUT_ROW_COUNT as i32),
            None
        );
        assert_eq!(
            preview_model_update_layout_preview_value(0x221),
            Some(u32::MAX)
        );
        assert_eq!(preview_model_update_static_flag(u32::MAX), None);
    }

    #[test]
    fn scene_list_rebuild_hook_targets_post_builder_flag_window() {
        assert_eq!(COSTUME_SCENE_LIST_BUILD_RVA, 0x1493220);
        assert_eq!(COSTUME_SCENE_LIST_BUILD_STOLEN_LEN, 20);
        assert!(COSTUME_SCENE_LIST_BUILD_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_RVA, 0x1493820);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_STOLEN_LEN, 14);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_BANK_STARTS_OFFSET, 0x2c0);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_COUNT_OFFSET, 0x2e0);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_SELECTED_LAYOUT_OFFSET, 0x2e4);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_BANK_OFFSET, 0x2e8);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_ROW_OFFSET, 0x2ec);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_INDEX_OFFSET, 0x2f0);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_OWNER_OFFSET, 0x2f8);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_WIDGET_OFFSET, 0x300);
        assert_eq!(COSTUME_SCENE_LIST_REBUILD_FLAG_OFFSET, 0x308);
        assert_eq!(
            COSTUME_SCENE_LIST_REBUILD_PARAM2_SOURCE_SLOT_OFFSET,
            7 * size_of::<usize>()
        );
        assert_eq!(
            scene_state_from_list_base(0x5000 + COSTUME_SCENE_LIST_BASE_OFFSET),
            Some(0x5000)
        );
        assert_eq!(scene_state_from_list_base(0x80), None);
    }

    #[test]
    fn costume_object_update_hook_targets_private_model_request_caller() {
        assert_eq!(COSTUME_OBJECT_UPDATE_RVA, 0x1498580);
        assert_eq!(COSTUME_OBJECT_UPDATE_STOLEN_LEN, 16);
        assert!(COSTUME_OBJECT_UPDATE_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
        assert_eq!(COSTUME_OBJECT_UPDATE_COUNT_OFFSET, 0x170);
        assert_eq!(COSTUME_OBJECT_UPDATE_SELECTED_INDEX_OFFSET, 0x174);
    }

    #[test]
    fn costume_object_apply_ready_hook_targets_post_model_ready_setter() {
        assert_eq!(COSTUME_OBJECT_APPLY_READY_RVA, 0x1494c20);
        assert_eq!(COSTUME_OBJECT_APPLY_READY_STOLEN_LEN, 15);
        assert!(COSTUME_OBJECT_APPLY_READY_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
    }

    #[test]
    fn costume_object_model_ready_check_hook_targets_slot_model_probe() {
        assert_eq!(COSTUME_OBJECT_MODEL_READY_CHECK_RVA, 0x135b7c0);
        assert_eq!(COSTUME_OBJECT_MODEL_READY_CHECK_STOLEN_LEN, 15);
        assert!(COSTUME_OBJECT_MODEL_READY_CHECK_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
    }

    #[test]
    fn costume_object_refresh_preview_hook_targets_post_busy_refresh() {
        assert!(!COSTUME_OBJECT_REFRESH_PREVIEW_HOOK_ENABLED);
        assert_eq!(COSTUME_OBJECT_REFRESH_PREVIEW_RVA, 0x1497f50);
        assert_eq!(COSTUME_OBJECT_REFRESH_PREVIEW_STOLEN_LEN, 17);
        assert!(COSTUME_OBJECT_REFRESH_PREVIEW_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
    }

    #[test]
    fn costume_object_refresh_preview_primary_callsite_targets_post_busy_refresh() {
        assert!(!COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_HOOK_ENABLED);
        assert!(!COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_NEXT_CALLSITE_ENABLED);
        assert!(!COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_PREV_CALLSITE_ENABLED);
        assert_eq!(
            COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA,
            0x14986ce
        );
        assert_eq!(
            COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_NEXT_CALLSITE_RVA,
            0x1498aab
        );
        assert_eq!(
            COSTUME_OBJECT_REFRESH_PREVIEW_SECONDARY_PREV_CALLSITE_RVA,
            0x1498bc4
        );
        assert_eq!(
            COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_BYTES,
            [0xe8, 0x7d, 0xf8, 0xff, 0xff]
        );
        assert_eq!(
            relative_call_bytes(0x1414986ce, 0x141497f50),
            Some(COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_BYTES)
        );
    }

    #[test]
    fn costume_object_update_trace_filter_keeps_custom_slot_and_private_model() {
        let base = CostumeObjectUpdateTrace {
            count: Some(5),
            selected_index: Some(0),
            mode: Some(26),
            cached_layout: Some(60),
            refresh_flag: Some(0),
            locked_flag: Some(0),
            object: Some(0x1000),
            controller: Some(0x2000),
            object_child: Some(0x3000),
            object_pending: Some(0),
            controller_model_loader: Some(0x4000),
            selected_layout: Some(60),
            selected_slot: Some(0),
            selected_kind: Some(0),
            selected_load_arg0: Some(0),
            selected_load_arg1: Some(0),
            selected_load_arg2: Some(0),
            selected_variant: Some(60),
            selected_model_resource: Some(26),
            selected_preview_mapping: Some(26),
            selected_preview_mapped_resource: Some(643),
        };

        assert!(!is_interesting_costume_object_update_trace(base));
        assert!(is_interesting_costume_object_update_trace(
            CostumeObjectUpdateTrace {
                selected_slot: Some(4),
                ..base
            }
        ));
        assert!(is_interesting_costume_object_update_trace(
            CostumeObjectUpdateTrace {
                selected_variant: Some(LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID),
                ..base
            }
        ));
        assert!(is_interesting_costume_object_update_trace(
            CostumeObjectUpdateTrace {
                selected_model_resource: Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                ..base
            }
        ));
    }

    #[test]
    fn law_ready_timeline_trace_labels_only_oni_and_custom_law_slots() {
        let _lock = HOOKS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let _probe_variant = set_atomic_usize_for_test(&LAW_EXTRA_SLOT_PROBE_VARIANT_ID, 699);
        let base = CostumeObjectUpdateTrace {
            count: Some(5),
            selected_index: Some(0),
            mode: Some(26),
            cached_layout: Some(u32::MAX),
            refresh_flag: Some(0),
            locked_flag: Some(0),
            object: Some(0x1000),
            controller: Some(0x2000),
            object_child: Some(0x3000),
            object_pending: Some(0),
            controller_model_loader: Some(0x4000),
            selected_layout: Some(u32::from(LAW_MASTER_LAYOUT_ID)),
            selected_slot: Some(0),
            selected_kind: Some(0),
            selected_load_arg0: Some(26),
            selected_load_arg1: Some(u32::from(u16::MAX)),
            selected_load_arg2: Some(0),
            selected_variant: Some(57),
            selected_model_resource: Some(26),
            selected_preview_mapping: Some(643),
            selected_preview_mapped_resource: Some(643),
        };

        assert_eq!(law_ready_timeline_costume_trace_label(base), None);
        assert_eq!(
            law_ready_timeline_costume_trace_label(CostumeObjectUpdateTrace {
                selected_slot: Some(3),
                selected_load_arg0: Some(308),
                selected_variant: Some(LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID),
                selected_model_resource: Some(308),
                selected_preview_mapping: Some(294),
                selected_preview_mapped_resource: Some(911),
                ..base
            }),
            Some("slot3-oni")
        );
        assert_eq!(
            law_ready_timeline_costume_trace_label(CostumeObjectUpdateTrace {
                selected_slot: Some(4),
                selected_load_arg0: Some(u32::from(
                    LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID
                )),
                selected_variant: Some(699),
                selected_model_resource: Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                selected_preview_mapping: Some(294),
                selected_preview_mapped_resource: Some(911),
                ..base
            }),
            Some("slot5-custom")
        );
    }

    #[test]
    fn late_model_ready_gate_requires_finalized_private_model_selection() {
        let base = CostumeObjectUpdateTrace {
            count: Some(5),
            selected_index: Some(LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32),
            mode: Some(70),
            cached_layout: Some(u32::MAX),
            refresh_flag: Some(0),
            locked_flag: Some(0),
            object: Some(0x1000),
            controller: Some(0x2000),
            object_child: Some(0x3000),
            object_pending: Some(0),
            controller_model_loader: Some(0x4000),
            selected_layout: Some(u32::from(LAW_MASTER_LAYOUT_ID)),
            selected_slot: Some(LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32),
            selected_kind: Some(0),
            selected_load_arg0: Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)),
            selected_load_arg1: Some(u32::from(u16::MAX)),
            selected_load_arg2: Some(0),
            selected_variant: Some(699),
            selected_model_resource: Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
            selected_preview_mapping: Some(294),
            selected_preview_mapped_resource: Some(911),
        };

        assert!(costume_object_update_trace_has_finalized_law_private_model(
            base
        ));
        assert!(
            !costume_object_update_trace_has_finalized_law_private_model(
                CostumeObjectUpdateTrace {
                    object_pending: Some(1),
                    ..base
                }
            )
        );
        assert!(
            !costume_object_update_trace_has_finalized_law_private_model(
                CostumeObjectUpdateTrace {
                    selected_load_arg0: Some(u32::from(
                        LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID
                    )),
                    selected_model_resource: Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID),
                    ..base
                }
            )
        );
    }

    #[test]
    fn law_private_model_ready_uses_flag_patch_not_result_override() {
        assert!(!LAW_EXTRA_SLOT_MODEL_READY_OVERRIDE_ENABLED);
        assert!(!LAW_EXTRA_SLOT_LATE_MODEL_READY_OVERRIDE_ENABLED);
        assert!(!LAW_EXTRA_SLOT_MODEL_READY_FLAG_PATCH_ENABLED);
        assert_eq!(
            format_model_ready_hook_detail(Some("flag_patch=true".to_string()), None).as_deref(),
            Some("flag_patch=true")
        );
    }

    #[test]
    fn model_render_attach_logging_has_global_probe_before_private_resource() {
        assert_eq!(
            model_render_attach_log_scope(0, Some(0), None),
            Some(ModelRenderAttachLogScope::Global)
        );
        assert_eq!(
            model_render_attach_log_scope(0, Some(MAX_MODEL_RENDER_ATTACH_GLOBAL_LOGS), None),
            None
        );
        assert_eq!(
            model_render_attach_log_scope(0x1234, None, Some(0)),
            Some(ModelRenderAttachLogScope::Private)
        );
        assert_eq!(
            model_render_attach_log_scope(0x1234, None, Some(MAX_MODEL_RENDER_ATTACH_LOGS)),
            None
        );
    }

    #[test]
    fn law_private_model_pointer_is_remembered_from_get_result_or_mirror() {
        let mirror = ModelResourceSlotMirror {
            source_pointer: 0x2222,
            source_state: 2,
            before_pointer: 0,
            before_state: 1,
            after_pointer: 0x2222,
            after_state: 2,
            write_attempted: true,
            write_ok: true,
            after_matches: true,
        };

        assert_eq!(
            law_private_model_pointer_to_remember(292, 0x1111, Some(mirror)),
            Some(0x1111)
        );
        assert_eq!(
            law_private_model_pointer_to_remember(292, 0, Some(mirror)),
            Some(0x2222)
        );
        assert_eq!(
            law_private_model_pointer_to_remember(26, 0x1111, Some(mirror)),
            None
        );
        assert_eq!(law_private_model_pointer_to_remember(292, 0, None), None);
    }

    #[test]
    fn private_model_name_patch_pads_shorter_target_to_registry_slot_capacity() {
        let patch = padded_nul_terminated_name_patch(b"MPLC000_Luffy\0", "MPLC026_Law")
            .expect("patch fits");

        assert_eq!(patch, b"MPLC026_Law\0\0\0");
    }

    #[test]
    fn entry35_base_from_source_hit_uses_model_row_stride() {
        let source_address = 0x5000 + entry35_row_start(26).expect("row address");

        assert_eq!(
            entry35_base_from_source_address(source_address, 26),
            Some(0x5000)
        );
    }

    #[test]
    fn extracts_file_name_from_wide_path() {
        let mut path: Vec<u16> = "D:\\Game\\OPPW4_PATCHER\\CharacterEditor\\MDLC038_Zoro_Wa.g1m"
            .encode_utf16()
            .collect();
        path.push(0);

        let file_name = wide_path_to_string(path.as_ptr()).and_then(|text| {
            Path::new(&text)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        });

        assert_eq!(file_name.as_deref(), Some("MDLC038_Zoro_Wa.g1m"));
    }

    #[test]
    fn tracks_rdb_and_rdb_bin_file_names() {
        assert_eq!(
            tracked_archive_name("CharacterEditor.rdb"),
            Some(("CharacterEditor".to_string(), TrackedFileKind::Index))
        );
        assert_eq!(
            tracked_archive_name("MaterialEditor.rdb.bin"),
            Some(("MaterialEditor".to_string(), TrackedFileKind::Data))
        );
        assert_eq!(
            tracked_archive_name("ScreenLayout.rdb.bin10"),
            Some(("ScreenLayout".to_string(), TrackedFileKind::Data))
        );
        assert_eq!(tracked_archive_name("not-an-archive.bin"), None);
    }

    #[test]
    fn classifies_dlc_linkdata_and_save_paths_for_stack_tracing() {
        assert_eq!(
            dlc_stack_path_kind(r"D:\Game\OPPW4\FILE\DLC\DLC_COSTUME_006_586_026_003.bin"),
            Some("dlc_costume")
        );
        assert_eq!(
            dlc_stack_path_kind(r"D:\Game\OPPW4\FILE\DLC\DLC_CHARACTER_021_975.bin"),
            Some("dlc_character_law_pack")
        );
        assert_eq!(
            dlc_stack_path_kind(r"D:\Game\OPPW4\FILE\DLC\DLC_CHARACTER_000_060.bin"),
            None
        );
        assert_eq!(
            dlc_stack_path_kind(r"D:\Game\OPPW4\FILE\DLC\DLC_UNKNOWN.bin"),
            None
        );
        assert_eq!(
            dlc_stack_path_kind(r"D:\Game\OPPW4\LINKDATA\CMN\LINKDATA_A.BIN"),
            Some("linkdata_a")
        );
        assert_eq!(dlc_stack_path_kind(r"C:\Save\OP4WINUSER.dat"), None);
        assert_eq!(dlc_stack_path_kind(r"D:\Game\OPPW4\File\CMN\foo.rdb"), None);
    }

    #[test]
    fn formats_menu_state_trace_with_selection_fields() {
        let trace = MenuStateTrace {
            mode: Some(2),
            category_mode: Some(2),
            current_row: Some(70),
            slot_index: Some(4),
            row_override: Some(0x7fff_ffff),
            list_index: Some(1),
            visible_count: Some(33),
            selected_index: Some(5),
            selected_row: Some(70),
            selected_layout: Some(26),
            selected_variant: Some(586),
            focus_variant: Some(586),
        };

        assert_eq!(
            format_menu_state_trace(trace),
            "mode=2 category_mode=2 current_row=70 slot_index=4 row_override=2147483647 list_index=1 count=33 selected_index=5 selected_row=70 selected_layout=26 selected_variant=586 focus_variant=586"
        );
    }

    #[test]
    fn formats_costume_scene_trace_with_selection_object_fields() {
        let trace = CostumeSceneTrace {
            layout: Some(26),
            row: Some(70),
            mode: Some(2),
            selected_variant: Some(586),
            selected_slot: Some(4),
            list_bank: Some(1),
            list_index: Some(3),
            refresh_flag: Some(1),
            special_flag: Some(0),
            locked_flag: Some(0),
            selected_object: Some(0x1234),
            object_layout: Some(26),
            object_variant: Some(586),
            object_slot: Some(4),
        };

        assert_eq!(
            format_costume_scene_trace(trace),
            "layout=26 row=70 mode=2 selected_variant=586 selected_slot=4 list_bank=1 list_index=3 refresh=0x01 special=0x00 locked=0x00 object=0x1234 object_layout=26 object_variant=586 object_slot=4"
        );
    }

    #[test]
    fn custom_allocated_variant_is_law_menu_value_for_trace_filters() {
        let _lock = HOOKS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let _probe_variant = set_atomic_usize_for_test(&LAW_EXTRA_SLOT_PROBE_VARIANT_ID, 699);

        assert!(is_law_menu_value(699));
        assert!(is_law_menu_value(586));
        assert!(!is_law_menu_value(700));
    }

    #[test]
    fn custom_scene_trace_after_regular_cap_keeps_reserved_diagnostics_available() {
        let _lock = HOOKS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let _regular_logs =
            set_atomic_usize_for_test(&COSTUME_SCENE_TRACE_LOGS, MAX_COSTUME_SCENE_TRACE_LOGS);
        let _custom_active = set_atomic_bool_for_test(&LAW_EXTRA_SLOT_CUSTOM_ACTIVE, true);
        let _probe_variant = set_atomic_usize_for_test(&LAW_EXTRA_SLOT_PROBE_VARIANT_ID, 699);
        let trace = CostumeSceneTrace {
            layout: Some(26),
            row: Some(70),
            mode: Some(2),
            selected_variant: Some(699),
            selected_slot: Some(4),
            list_bank: Some(1),
            list_index: Some(3),
            refresh_flag: Some(1),
            special_flag: Some(0),
            locked_flag: Some(0),
            selected_object: None,
            object_layout: None,
            object_variant: None,
            object_slot: None,
        };

        log_costume_scene_state_call(
            "scene-update-dispatcher",
            0x1234,
            String::new(),
            Some(trace),
            Some(trace),
            false,
        );

        assert_eq!(
            COSTUME_SCENE_TRACE_LOGS.load(Ordering::Relaxed),
            MAX_COSTUME_SCENE_TRACE_LOGS
        );
    }

    #[test]
    fn law_custom_scene_available_diagnostic_noops_while_disabled() {
        let mut trace = CostumeSceneTrace {
            layout: Some(26),
            row: Some(70),
            mode: Some(2),
            selected_variant: Some(699),
            selected_slot: Some(4),
            list_bank: Some(1),
            list_index: Some(3),
            refresh_flag: Some(1),
            special_flag: Some(0),
            locked_flag: Some(0),
            selected_object: Some(0x1234),
            object_layout: Some(26),
            object_variant: Some(699),
            object_slot: Some(4),
        };

        assert_eq!(
            law_custom_scene_available_check_result(0, Some(trace), None, Some(699)),
            (0, false)
        );
        assert_eq!(
            law_custom_scene_available_check_result(1, Some(trace), None, Some(699)),
            (1, false)
        );

        trace.selected_variant = Some(586);
        trace.object_variant = Some(586);
        assert_eq!(
            law_custom_scene_available_check_result(0, Some(trace), None, Some(699)),
            (0, false)
        );

        trace.selected_variant = Some(699);
        trace.object_variant = Some(699);
        trace.layout = Some(25);
        assert_eq!(
            law_custom_scene_available_check_result(0, Some(trace), None, Some(699)),
            (0, false)
        );
    }

    #[test]
    fn preview_model_trace_keeps_custom_budget_after_global_cap() {
        assert_eq!(
            preview_model_update_trace_scope(
                false,
                true,
                false,
                0,
                MAX_COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS,
                0
            ),
            None
        );
        assert_eq!(
            preview_model_update_trace_scope(
                false,
                true,
                true,
                0,
                MAX_COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS,
                0
            ),
            Some(PreviewModelUpdateTraceScope::Custom)
        );
        assert_eq!(
            preview_model_update_trace_scope(
                false,
                true,
                true,
                0,
                MAX_COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS,
                MAX_COSTUME_PREVIEW_MODEL_CUSTOM_TRACE_LOGS
            ),
            None
        );
        assert_eq!(
            preview_model_update_trace_scope(
                true,
                true,
                true,
                0,
                MAX_COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS,
                MAX_COSTUME_PREVIEW_MODEL_CUSTOM_TRACE_LOGS
            ),
            Some(PreviewModelUpdateTraceScope::Slot5)
        );
    }

    #[test]
    fn formats_preview_model_widget_trace_core_fields() {
        let trace = PreviewModelWidgetTrace {
            widget: 0x1000,
            child58: Some(0x2000),
            field10: Some(42),
            mapped294: Some(699),
            current_layout2dc: Some(26),
            visible2a0: Some(1),
            active2a1: Some(1),
            child_b4: Some(0x123),
            child_28: Some(0x3000),
            child_30: Some(0x4000),
            child_38: Some(0x5000),
            child_48: None,
        };

        assert_eq!(
            format_preview_model_widget_trace(trace),
            "widget=0x1000 child58=0x2000 field10=42 mapped294=699 current_layout2dc=26 visible2a0=0x01 active2a1=0x01 child_b4=291 child28=0x3000 child30=0x4000 child38=0x5000 child48=none"
        );
    }

    #[test]
    fn formats_preview_model_child_entry_probe_flags() {
        assert_eq!(
            format_preview_model_child_entry_text(
                6,
                0x2000,
                Some(0x0d),
                Some(1),
                Some(0x3f800000),
                Some(0),
                None
            ),
            "6:obj=0x2000,flags30=0x0000000d,b112=0x01,v34=0x3f800000,v3c=0x00000000,v44=none"
        );
    }

    #[test]
    fn formats_launch_costume_state_trace_key_validation_fields() {
        let trace = LaunchCostumeStateTrace {
            state: 0x1000,
            index4: Some(2),
            flags20: Some(0x2002),
            id_1d0: Some(699),
            id_1d4: Some(26),
            id_1d8: Some(0xffff),
            id_1dc: Some(4),
            id_1e4: Some(57),
            ptr_f0: Some(0x2000),
            ptr_f8: Some(0x3000),
            ptr_430: Some(0x4000),
            ptr_430_7bd8: Some(0x5000),
            object_14d: Some(0xff),
            object_1dc: Some(0x10),
            object_232: Some(3),
            object_235: Some(0),
        };

        assert_eq!(
            format_launch_costume_state_trace(trace),
            "state=0x1000 index4=2 flags20=0x00002002 id1d0=699 id1d4=26 id1d8=65535 id1dc=4 id1e4=57 ptr_f0=0x2000 ptr_f8=0x3000 ptr430=0x4000 ptr430_7bd8=0x5000 obj14d=0xff obj1dc=16 obj232=0x03 obj235=0x00"
        );
    }

    #[test]
    fn law_launch_private_model_alias_maps_only_private_model_id_for_slot4() {
        let mut trace = LaunchCostumeStateTrace {
            state: 0x1000,
            index4: Some(0xff),
            flags20: Some(0x1043),
            id_1d0: Some(292),
            id_1d4: Some(22),
            id_1d8: Some(0xffff),
            id_1dc: Some(0),
            id_1e4: Some(4),
            ptr_f0: Some(0x2000),
            ptr_f8: Some(0x3000),
            ptr_430: Some(0x4000),
            ptr_430_7bd8: Some(0x5000),
            object_14d: Some(0xff),
            object_1dc: Some(0xffff),
            object_232: Some(0),
            object_235: Some(0),
        };

        assert_eq!(law_launch_state_private_model_alias_target(trace), Some(26));

        trace.id_1d0 = Some(26);
        assert_eq!(law_launch_state_private_model_alias_target(trace), None);

        trace.id_1d0 = Some(292);
        trace.id_1e4 = Some(3);
        assert_eq!(law_launch_state_private_model_alias_target(trace), None);
    }

    #[test]
    fn law_custom_preview_visible_diagnostic_stays_disabled_after_character_lock_regression() {
        assert_eq!(
            law_custom_preview_model_visible_arg(
                699,
                Some(699),
                0,
                i32::from(LAW_MASTER_LAYOUT_ID)
            ),
            (0, false)
        );
        assert_eq!(
            law_custom_preview_model_visible_arg(
                699,
                Some(699),
                1,
                i32::from(LAW_MASTER_LAYOUT_ID)
            ),
            (1, false)
        );
        assert_eq!(
            law_custom_preview_model_visible_arg(
                586,
                Some(699),
                0,
                i32::from(LAW_MASTER_LAYOUT_ID)
            ),
            (0, false)
        );
        assert_eq!(
            law_custom_preview_model_visible_arg(
                699,
                Some(699),
                0,
                i32::from(LAW_MASTER_LAYOUT_ID + 1)
            ),
            (0, false)
        );
    }

    #[test]
    fn law_custom_conditional_preview_diagnostic_stays_disabled_after_negative_probe() {
        assert!(!LAW_EXTRA_SLOT_FORCE_CONDITIONAL_PREVIEW_DIAGNOSTIC_ENABLED);
        assert!(!law_custom_preview_conditional_branch_diagnostic(
            699,
            Some(699),
            0,
            i32::from(LAW_MASTER_LAYOUT_ID),
            0
        ));
        assert!(!law_custom_preview_conditional_branch_diagnostic(
            699,
            Some(699),
            1,
            i32::from(LAW_MASTER_LAYOUT_ID),
            0
        ));
        assert!(!law_custom_preview_conditional_branch_diagnostic(
            586,
            Some(699),
            0,
            i32::from(LAW_MASTER_LAYOUT_ID),
            0
        ));
        assert!(!law_custom_preview_conditional_branch_diagnostic(
            699,
            Some(699),
            0,
            25,
            0
        ));
        assert!(!law_custom_preview_conditional_branch_diagnostic(
            699,
            Some(699),
            0,
            i32::from(LAW_MASTER_LAYOUT_ID),
            1
        ));
    }

    #[test]
    fn scene_list_flag_slot_uses_current_index_before_layout_scan() {
        let layouts = [26, 49, 50, 45, 14, 11, 27, 15, 10, 13, 0, 0, 0, 0];

        assert_eq!(scene_list_flag_slot_for_preview(0, 26, &layouts), Some(0));
        assert_eq!(scene_list_flag_slot_for_preview(4, 26, &layouts), Some(4));
    }

    #[test]
    fn scene_list_flag_slot_scans_layout_when_index_is_out_of_entry_range() {
        let layouts = [49, 50, 26, 45, 14, 11, 27, 15, 10, 13, 0, 0, 0, 0];

        assert_eq!(
            scene_list_flag_slot_for_preview(
                COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT as u32,
                26,
                &layouts
            ),
            Some(2)
        );
        assert_eq!(scene_list_flag_slot_for_preview(99, 26, &layouts), Some(2));
        assert_eq!(scene_list_flag_slot_for_preview(99, 999, &layouts), None);
    }

    #[test]
    fn preview_visible_decision_uses_direct_index_gate_first() {
        let layouts = [26, 49, 50, 45, 14, 11, 27, 15, 10, 13, 0, 0, 0, 0];
        let mut flags = [0u8; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT];
        flags[4] = 1;

        let decision = costume_preview_visible_decision(4, 26, &layouts, &flags);

        assert_eq!(decision.direct_slot, Some(4));
        assert_eq!(decision.layout_slot, Some(0));
        assert_eq!(decision.decision_slot, Some(4));
        assert_eq!(decision.decision_flag, Some(1));
        assert_eq!(decision.decision_rule(), "direct-index");
        assert!(decision.would_call_busy_check());
    }

    #[test]
    fn preview_visible_decision_scans_layout_when_index_is_out_of_entry_range() {
        let layouts = [49, 50, 26, 45, 14, 11, 27, 15, 10, 13, 0, 0, 0, 0];
        let mut flags = [0u8; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT];
        flags[2] = 1;

        let decision = costume_preview_visible_decision(99, 26, &layouts, &flags);

        assert_eq!(decision.direct_slot, None);
        assert_eq!(decision.layout_slot, Some(2));
        assert_eq!(decision.decision_slot, Some(2));
        assert_eq!(decision.decision_flag, Some(1));
        assert_eq!(decision.decision_rule(), "layout-scan");
        assert_eq!(decision.flag_gate(), "flag-open");
    }

    #[test]
    fn law_custom_scene_list_flag_patch_requires_current_custom_variant() {
        let mut trace = CostumeSceneTrace {
            layout: Some(26),
            row: Some(70),
            mode: Some(2),
            selected_variant: Some(699),
            selected_slot: Some(4),
            list_bank: Some(2),
            list_index: Some(0),
            refresh_flag: Some(0),
            special_flag: Some(0),
            locked_flag: Some(1),
            selected_object: Some(0x1234),
            object_layout: Some(26),
            object_variant: Some(586),
            object_slot: Some(3),
        };

        assert!(law_custom_scene_list_flag_patch_candidate(trace, Some(699)));
        assert!(!law_custom_scene_list_flag_patch_candidate(
            trace,
            Some(700)
        ));

        trace.selected_variant = Some(586);
        trace.object_variant = Some(699);
        trace.object_slot = Some(4);
        assert!(law_custom_scene_list_flag_patch_candidate(trace, Some(699)));

        trace.layout = Some(25);
        assert!(!law_custom_scene_list_flag_patch_candidate(
            trace,
            Some(699)
        ));
        assert!(!law_custom_scene_list_flag_patch_candidate(trace, None));
    }

    #[test]
    fn law_custom_scene_locked_flag_patch_requires_current_custom_locked_state() {
        let mut trace = CostumeSceneTrace {
            layout: Some(26),
            row: Some(70),
            mode: Some(2),
            selected_variant: Some(699),
            selected_slot: Some(4),
            list_bank: Some(2),
            list_index: Some(0),
            refresh_flag: Some(0),
            special_flag: Some(0),
            locked_flag: Some(1),
            selected_object: Some(0x1234),
            object_layout: Some(26),
            object_variant: Some(586),
            object_slot: Some(3),
        };

        assert!(law_custom_scene_locked_flag_patch_candidate(
            trace,
            Some(699)
        ));

        trace.locked_flag = Some(0);
        assert!(!law_custom_scene_locked_flag_patch_candidate(
            trace,
            Some(699)
        ));

        trace.locked_flag = Some(1);
        trace.selected_variant = Some(586);
        trace.object_variant = Some(699);
        trace.object_slot = Some(4);
        assert!(law_custom_scene_locked_flag_patch_candidate(
            trace,
            Some(699)
        ));

        trace.layout = Some(25);
        assert!(!law_custom_scene_locked_flag_patch_candidate(
            trace,
            Some(699)
        ));
        assert!(!law_custom_scene_locked_flag_patch_candidate(trace, None));
    }

    #[test]
    fn law_custom_scene_locked_flag_patch_stays_disabled_after_character_lock_regression() {
        assert!(!LAW_EXTRA_SLOT_CLEAR_SCENE_LOCKED_DIAGNOSTIC_ENABLED);
    }

    #[test]
    fn law_custom_scene_available_force_stays_disabled_after_character_lock_regression() {
        assert!(!LAW_EXTRA_SLOT_FORCE_SCENE_AVAILABLE_DIAGNOSTIC_ENABLED);
    }

    #[test]
    fn law_custom_scene_list_flag_patch_stays_disabled_after_character_lock_regression() {
        assert!(!LAW_EXTRA_SLOT_SCENE_LIST_FLAG_DIAGNOSTIC_ENABLED);
    }

    #[test]
    fn law_extra_slot_probe_uses_unique_variant_for_slot5_test() {
        assert_eq!(LAW_EXTRA_SLOT_PROBE_FALLBACK_VARIANT_ID, 587);
        assert_ne!(
            LAW_EXTRA_SLOT_PROBE_FALLBACK_VARIANT_ID,
            costume_table::LAW_DUPLICATE_VARIANT_ID
        );
    }

    #[test]
    fn law_extra_slot_allocator_uses_free_candidate_and_skips_owned_probe_id() {
        let dump = layout_dump_for_allocation_test(
            vec![85, 587, 703, 728],
            vec![CostumeVariantOwner {
                variant_id: 587,
                layout_id: 74,
                slot_index: 1,
                family: 126,
                category: 74,
                active_variant_count: 2,
            }],
        );

        let allocation = choose_law_extra_slot_allocation(&dump).expect("allocation");

        assert_eq!(allocation.character, "Law");
        assert_eq!(
            allocation.source_variant_id,
            costume_table::LAW_DUPLICATE_VARIANT_ID
        );
        assert_eq!(
            allocation.metadata_source_layout_id,
            LAW_EXTRA_SLOT_METADATA_SOURCE_LAYOUT_ID
        );
        assert_eq!(
            allocation.metadata_source_variant_id,
            LAW_EXTRA_SLOT_METADATA_SOURCE_VARIANT_ID
        );
        assert_eq!(allocation.allocated_variant_id, 703);
        assert_eq!(allocation.slot_index, LAW_DUPLICATE_VARIANT_SLOT_INDEX);
        assert_eq!(allocation.mode, "auto-free-layout");
        assert_eq!(
            format_custom_variant_allocations(&[allocation]),
            "Law slot=4 source=586 metadata_layout=131 metadata_variant=586 allocated=703 mode=auto-free-layout"
        );
    }

    #[test]
    fn law_extra_slot_allocator_rejects_candidates_above_game_selectable_limit() {
        let dump = layout_dump_for_allocation_test(vec![704, 724, 728], Vec::new());

        assert_eq!(choose_law_extra_slot_allocation(&dump), None);
    }

    #[test]
    fn variant_metadata_record_address_uses_game_variant_table_stride() {
        let base = 0x3000;

        assert_eq!(
            costume_variant_metadata_record_address(base, costume_table::LAW_DUPLICATE_VARIANT_ID),
            Some(
                base + COSTUME_VARIANT_METADATA_BASE_OFFSET
                    + usize::from(costume_table::LAW_DUPLICATE_VARIANT_ID)
                        * COSTUME_VARIANT_METADATA_STRIDE
            )
        );
        assert_eq!(costume_variant_metadata_record_address(base, 704), None);
    }

    #[test]
    fn variant_metadata_clone_includes_presentation_fields_before_unlock_flags() {
        assert_eq!(COSTUME_VARIANT_METADATA_BASE_OFFSET, 0xd92c);
        assert_eq!(
            COSTUME_VARIANT_METADATA_BASE_OFFSET + COSTUME_VARIANT_METADATA_FLAGS_OFFSET,
            0xd940
        );
    }

    #[test]
    fn variant_metadata_material_patch_targets_color_fields_before_unlock_flags() {
        assert_eq!(
            LAW_EXTRA_SLOT_COLOR_VARIATION_SOURCE_VARIANT_ID,
            costume_table::LAW_DUPLICATE_VARIANT_ID
        );
        assert_eq!(COSTUME_VARIANT_METADATA_COLOR_VARIATION_OFFSET, 0x0c);
        assert_eq!(COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE, 0x08);
        assert!(
            COSTUME_VARIANT_METADATA_COLOR_VARIATION_OFFSET
                + COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE
                <= COSTUME_VARIANT_METADATA_FLAGS_OFFSET
        );
    }

    #[test]
    fn variant_metadata_preview_mapping_field_matches_preview_mapper_read() {
        assert_eq!(COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET, 0x06);
        assert_eq!(
            COSTUME_VARIANT_METADATA_BASE_OFFSET + COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET,
            0xd932
        );
        assert_eq!(COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE, 2);
        assert!(
            COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET
                + COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE
                <= COSTUME_VARIANT_METADATA_COLOR_VARIATION_OFFSET
        );
    }

    #[test]
    fn law_preview_mapping_diagnostic_uses_oni_preview_resource_id() {
        assert_eq!(LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID, 586);
        assert_eq!(preview_model_mapped_id_from_metadata_value(26), 643);
        assert_eq!(preview_model_mapped_id_from_metadata_value(294), 911);
    }

    #[test]
    fn runtime_category_slot_flag_address_uses_unlock_table_shape() {
        let runtime_root = 0x5000;
        let slot_base = 0x9000;

        assert_eq!(
            runtime_category_slot_pointer_address(runtime_root, LAW_MASTER_CATEGORY_ID as u16),
            Some(runtime_root + (26 + COSTUME_RUNTIME_CATEGORY_SLOT_BASE) * size_of::<usize>())
        );
        assert_eq!(
            runtime_category_slot_flag_address(slot_base, LAW_DUPLICATE_VARIANT_SLOT_INDEX),
            Some(
                slot_base
                    + COSTUME_RUNTIME_UNLOCK_SLOT_FLAGS_OFFSET
                    + LAW_DUPLICATE_VARIANT_SLOT_INDEX
            )
        );
        assert_eq!(
            runtime_category_slot_pointer_address(runtime_root, COSTUME_CATEGORY_COUNT as u16),
            None
        );
        assert_eq!(
            runtime_category_slot_flag_address(slot_base, COSTUME_LAYOUT_VARIANT_COUNT as usize),
            None
        );
    }

    #[test]
    fn custom_slot_flags_avoid_official_dlc_entitlement_branch() {
        assert_eq!(custom_variant_metadata_flags(0x07), 0x05);
        assert_eq!(custom_variant_metadata_flags(0x02), 0x01);
        assert_eq!(custom_runtime_unlock_slot_flags(0x0a, true), 0x0b);
        assert_eq!(custom_runtime_unlock_slot_flags(0x0a, false), 0x0a);
    }

    #[test]
    fn law_extra_slot_metadata_clone_uses_oni_layout_as_source() {
        let mut dump = layout_dump_for_allocation_test(Vec::new(), Vec::new());
        dump.static_layouts = 0x3000;
        let allocation = CustomVariantAllocation {
            character: "Law",
            source_variant_id: costume_table::LAW_DUPLICATE_VARIANT_ID,
            metadata_source_layout_id: LAW_EXTRA_SLOT_METADATA_SOURCE_LAYOUT_ID,
            metadata_source_variant_id: LAW_EXTRA_SLOT_METADATA_SOURCE_VARIANT_ID,
            allocated_variant_id: 728,
            slot_index: LAW_DUPLICATE_VARIANT_SLOT_INDEX,
            mode: "auto-free-layout",
        };

        let clone = law_extra_slot_metadata_clone_for_allocation(&dump, allocation)
            .expect("metadata clone");

        assert_eq!(
            clone,
            CustomVariantMetadataClone {
                source_layout_id: LAW_EXTRA_SLOT_METADATA_SOURCE_LAYOUT_ID,
                target_layout_id: 728,
                source_address: 0x3000
                    + usize::from(LAW_EXTRA_SLOT_METADATA_SOURCE_LAYOUT_ID)
                        * COSTUME_LAYOUT_ROW_STRIDE,
                target_address: 0x3000 + 728 * COSTUME_LAYOUT_ROW_STRIDE,
                byte_count: COSTUME_LAYOUT_ROW_COPY_SIZE,
            }
        );
    }

    #[test]
    fn law_extra_slot_dynamic_dlc_file_alias_uses_allocated_id() {
        assert_eq!(
            law_extra_slot_probe_alias_path_for_variant(
                r"D:\Game\OPPW4\FILE\DLC\DLC_COSTUME_006_728_026_004.bin",
                728
            )
            .as_deref(),
            Some(r"D:\Game\OPPW4\FILE\DLC\DLC_COSTUME_006_586_026_003.bin")
        );
        assert_eq!(
            law_extra_slot_probe_alias_path_for_variant(
                r"D:\Game\OPPW4\FILE\DLC\DLC_COSTUME_006_587_026_004.bin",
                728
            ),
            None
        );
    }

    #[test]
    fn law_selection_lookup_override_noops_for_unique_slot_probe() {
        assert_eq!(
            law_selection_lookup_override(
                26,
                u32::from(LAW_MASTER_LAYOUT_ID),
                costume_table::LAW_DUPLICATE_VARIANT_ID
            ),
            None
        );
        assert_eq!(
            law_selection_lookup_override(26, u32::from(LAW_MASTER_LAYOUT_ID), 57),
            None
        );
        assert_eq!(
            law_selection_lookup_override(
                25,
                u32::from(LAW_MASTER_LAYOUT_ID),
                costume_table::LAW_DUPLICATE_VARIANT_ID
            ),
            None
        );
        assert_eq!(
            law_selection_lookup_override(26, 57, costume_table::LAW_DUPLICATE_VARIANT_ID),
            None
        );
    }

    #[test]
    fn selection_setter_hook_does_not_split_mov_r11_rcx() {
        assert_eq!(COSTUME_SELECTION_SETTER_RVA, 0x0b16c0);
        assert_eq!(COSTUME_SELECTION_SETTER_STOLEN_LEN, 16);
        assert!(COSTUME_SELECTION_SETTER_STOLEN_LEN >= ABSOLUTE_JUMP_LEN);
    }

    #[test]
    fn law_extra_slot_probe_unlock_stays_guarded_until_metadata_path_is_complete() {
        assert_eq!(
            law_extra_slot_probe_unlock_override_for_variant(26, 728, 0, 587),
            None
        );
        assert_eq!(
            law_extra_slot_probe_unlock_override_for_variant(26, 728, 1, 728),
            None
        );
        assert_eq!(
            law_extra_slot_probe_unlock_override_for_variant(25, 728, 0, 728),
            None
        );
        assert_eq!(
            law_extra_slot_probe_unlock_override_for_variant(26, 728, 0, 728),
            None
        );
    }

    #[test]
    fn law_extra_slot_probe_dlc_file_aliases_to_existing_law_oni_file() {
        assert_eq!(
            law_extra_slot_probe_alias_path_for_variant(
                r"D:\Game\OPPW4\FILE\DLC\DLC_COSTUME_006_587_026_004.bin",
                LAW_EXTRA_SLOT_PROBE_FALLBACK_VARIANT_ID
            )
            .as_deref(),
            Some(r"D:\Game\OPPW4\FILE\DLC\DLC_COSTUME_006_586_026_003.bin")
        );
        assert_eq!(
            law_extra_slot_probe_alias_path_for_variant(
                r"D:\Game\OPPW4\FILE\DLC\DLC_COSTUME_006_586_026_003.bin",
                LAW_EXTRA_SLOT_PROBE_FALLBACK_VARIANT_ID
            ),
            None
        );
    }

    #[test]
    fn formats_costume_variant_owners_for_probe_logs() {
        let owners = [
            CostumeVariantOwner {
                variant_id: 587,
                layout_id: 420,
                slot_index: 2,
                family: 12,
                category: 34,
                active_variant_count: 5,
            },
            CostumeVariantOwner {
                variant_id: 587,
                layout_id: 421,
                slot_index: 0,
                family: 56,
                category: 78,
                active_variant_count: 1,
            },
        ];

        assert_eq!(
            format_costume_variant_owners(&owners),
            "420:2(family=12,category=34,count=5),421:0(family=56,category=78,count=1)"
        );
    }

    fn layout_dump_for_allocation_test(
        free_layout_candidates: Vec<u16>,
        variant_owners: Vec<CostumeVariantOwner>,
    ) -> CostumeLayoutTableDump {
        CostumeLayoutTableDump {
            static_database: 0,
            static_root: 0,
            static_layouts: 0,
            static_rows: 0,
            runtime_database: None,
            runtime_root: None,
            layouts: Vec::new(),
            matrix_probes: Vec::new(),
            variant_owners,
            free_layout_candidates,
        }
    }

    #[test]
    fn law_variant_unlock_override_only_for_hidden_law_variant() {
        assert_eq!(
            law_variant_unlock_override(26, LAW_HIDDEN_VARIANT_ID, 0),
            Some(1)
        );
        assert_eq!(
            law_variant_unlock_override(26, LAW_HIDDEN_VARIANT_ID, 1),
            None
        );
        assert_eq!(
            law_variant_unlock_override(25, LAW_HIDDEN_VARIANT_ID, 0),
            None
        );
        assert_eq!(law_variant_unlock_override(26, 586, 0), None);
    }

    #[test]
    fn hidden_law_variant_unlock_is_controlled_by_its_own_flag() {
        set_duplicate_law_variant_slot(true);
        set_unlock_law_hidden_variant(false);

        assert_eq!(
            law_variant_unlock_override_if_enabled(26, LAW_HIDDEN_VARIANT_ID, 0, 0),
            None
        );

        set_unlock_law_hidden_variant(true);

        assert_eq!(
            law_variant_unlock_override_if_enabled(26, LAW_HIDDEN_VARIANT_ID, 0, 0),
            Some(1)
        );

        set_unlock_law_hidden_variant(false);
        set_duplicate_law_variant_slot(false);
    }

    #[test]
    fn layout_availability_hook_targets_fun_1412f92c0_without_trampoline() {
        assert_eq!(COSTUME_LAYOUT_AVAILABILITY_CHECK_RVA, 0x12f92c0);
        assert_eq!(
            COSTUME_LAYOUT_AVAILABILITY_CHECK_STOLEN_LEN,
            ABSOLUTE_JUMP_LEN
        );
        assert_eq!(COSTUME_LAYOUT_AVAILABILITY_TARGET_OFFSET, 0x14);
        assert_eq!(COSTUME_LAYOUT_AVAILABILITY_PREVIEW_OFFSET, 0x16);
        assert_eq!(COSTUME_LAYOUT_AVAILABILITY_FLAG_OFFSET, 0x4a);
        assert_eq!(COSTUME_LAYOUT_AVAILABILITY_TABLE_OFFSET, 0xc1b4);
        assert_eq!(COSTUME_LAYOUT_AVAILABILITY_TABLE_COUNT, 300);
        assert_eq!(COSTUME_LAYOUT_AVAILABILITY_TABLE_STRIDE, 2);
    }

    #[test]
    fn layout_availability_wrapper_must_preserve_internal_callsite_registers() {
        assert_eq!(COSTUME_SELECTION_SETTER_RVA, 0x0b16c0);
        assert_eq!(COSTUME_LAYOUT_AVAILABILITY_CHECK_RVA, 0x12f92c0);
        // FUN_1400b16c0 keeps R10/R11 live across its call to FUN_1412f92c0.
        // FUN_1414906a0 also keeps R9 live across the same internal helper.
        // The hook wrapper must preserve them even though they are volatile in
        // the Windows ABI.
    }

    #[test]
    fn law_scene_bank_layout_filter_keeps_current_law_bank_members() {
        for layout in [26, 49, 50, 45, 14, 11, 27, 15, 10, 13] {
            assert!(is_law_scene_bank_layout_id(layout));
        }
        for layout in [0, 57, 82, 292, 555, 699] {
            assert!(!is_law_scene_bank_layout_id(layout));
        }
    }
}
