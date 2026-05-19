use mlua::{Function, Lua, Table, Value};

use struct_api::Character;

use crate::{LuaMod, ModSource};

#[derive(Debug)]
pub enum LuaRunError {
    ReadScript(std::io::Error),
    Lua(mlua::Error),
}

pub fn run_lua_mod<F>(mod_entry: &LuaMod, register_modules: F) -> Result<(), LuaRunError>
where
    F: FnOnce(&Lua) -> mlua::Result<()>,
{
    let lua = Lua::new();
    install_runtime(&lua).map_err(LuaRunError::Lua)?;
    install_mod_globals(&lua, mod_entry).map_err(LuaRunError::Lua)?;
    register_modules(&lua).map_err(LuaRunError::Lua)?;
    let source = mod_entry
        .read_entry_script()
        .map_err(LuaRunError::ReadScript)?;
    lua.load(&source)
        .set_name(format!(
            "{}:{}",
            mod_entry.manifest.id, mod_entry.manifest.entry_lua
        ))
        .exec()
        .map_err(LuaRunError::Lua)
}

fn install_mod_globals(lua: &Lua, mod_entry: &LuaMod) -> mlua::Result<()> {
    let globals = lua.globals();
    globals.set("__oppw4_mod_id", mod_entry.manifest.id.as_str())?;
    globals.set("__oppw4_mod_name", mod_entry.manifest.name.as_str())?;
    globals.set(
        "__oppw4_mod_root",
        match &mod_entry.source {
            ModSource::Directory(root) => root.to_string_lossy().to_string(),
            ModSource::Zip { path, .. } => path.to_string_lossy().to_string(),
        },
    )?;
    globals.set(
        "__oppw4_mod_zip_root",
        match &mod_entry.source {
            ModSource::Directory(_) => String::new(),
            ModSource::Zip { root, .. } => root.clone(),
        },
    )?;
    globals.set(
        "__oppw4_mod_is_zip",
        matches!(mod_entry.source, ModSource::Zip { .. }),
    )
}

pub fn install_runtime(lua: &Lua) -> mlua::Result<()> {
    install_character_core(lua)?;
    install_require_hook(lua)
}

pub fn install_require_hook(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    let require: Function = globals.get("require")?;
    let imported = lua.create_table()?;
    globals.set(
        "require",
        lua.create_function(move |_, name: String| {
            let module: Value = require.call(name.as_str())?;
            let already_imported = imported
                .get::<Option<bool>>(name.as_str())?
                .unwrap_or(false);
            if !already_imported {
                imported.set(name.as_str(), true)?;
                if let Value::Table(table) = &module {
                    if let Some(on_import) = table.get::<Option<Function>>("__oppw4_on_import")? {
                        on_import.call::<()>(())?;
                    }
                }
            }
            Ok(module)
        })?,
    )
}

pub fn register_module(lua: &Lua, name: &str, table: Table) -> mlua::Result<()> {
    let package: Table = lua.globals().get("package")?;
    let preload: Table = package.get("preload")?;
    let key = lua.create_registry_value(table)?;
    preload.set(
        name,
        lua.create_function(move |lua, ()| lua.registry_value::<Table>(&key))?,
    )
}

fn install_character_core(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    let methods = lua.create_table()?;
    let owners = lua.create_table()?;
    globals.set("__struct_api_methods", methods.clone())?;
    globals.set("__struct_api_method_owners", owners.clone())?;

    globals.set(
        "__oppw4_register_character_method",
        lua.create_function(
            move |_, (owner, name, method): (String, String, Function)| {
                if let Some(existing_owner) = owners.get::<Option<String>>(name.as_str())? {
                    if !existing_owner.eq_ignore_ascii_case(&owner) {
                        return Err(mlua::Error::external(format!(
                            "character.{name} already registered by {existing_owner}, refused by {owner}"
                        )));
                    }
                }
                owners.set(name.as_str(), owner)?;
                methods.set(name, method)
            },
        )?,
    )?;

    let character = lua.create_table()?;
    character.set(
        "find",
        lua.create_function(|lua, query: Value| match query {
            Value::String(name) => {
                let Some(character) = struct_api::find(name.to_str()?.as_ref()) else {
                    return Ok(Value::Nil);
                };
                Ok(Value::Table(character_handle_table(lua, character)?))
            }
            Value::Integer(id) if (0..=u16::MAX as i64).contains(&id) => {
                let Some(character) = struct_api::find_by_id(id as u16) else {
                    return Ok(Value::Nil);
                };
                Ok(Value::Table(character_handle_table(lua, character)?))
            }
            _ => Ok(Value::Nil),
        })?,
    )?;
    character.set(
        "unsafe_find",
        lua.create_function(|lua, query: Value| unsafe_character_handle_table(lua, query))?,
    )?;
    character.set(
        "new",
        lua.create_function(|lua, fields: Table| custom_character_handle_table(lua, fields))?,
    )?;
    character.set(
        "all",
        lua.create_function(|lua, ()| {
            let rows = lua.create_table()?;
            for (index, character) in struct_api::all().iter().enumerate() {
                rows.set(index + 1, character_handle_table(lua, character)?)?;
            }
            Ok(rows)
        })?,
    )?;
    character.set(
        "local_player",
        lua.create_function(|lua, ()| local_player_handle_table(lua))?,
    )?;
    register_module(lua, "character", character.clone())?;
    globals.set("character", character)
}

fn character_handle_table(lua: &Lua, character: &Character) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("kind", "character")?;
    table.set("known", true)?;
    table.set("unsafe", false)?;
    if let Some(model_id) = character.model_id {
        table.set("id", model_id)?;
        table.set("model_id", model_id)?;
    }
    if let Some(playable_id) = character.playable_id {
        table.set("playable_id", playable_id)?;
    }
    if let Some(runtime_id) = character.runtime_id {
        table.set("runtime_id", runtime_id)?;
    }
    if let Some(boss_runtime_id) = character.boss_runtime_id {
        table.set("boss_runtime_id", boss_runtime_id)?;
    }
    if let Some(entry) = character.moveset_linkdata_entry {
        table.set("moveset_linkdata_entry", entry)?;
    }
    table.set("name", character.canonical.as_str())?;
    table.set("canonical", character.canonical.as_str())?;
    table.set("display_name", character.display_name.as_str())?;
    table.set("model_stem", character.model_stem.as_str())?;

    let metatable = lua.create_table()?;
    let methods: Table = lua.globals().get("__struct_api_methods")?;
    metatable.set(
        "__index",
        lua.create_function(move |_, (_this, key): (Table, String)| {
            methods.get::<Value>(key.as_str())
        })?,
    )?;
    table.set_metatable(Some(metatable));
    Ok(table)
}

#[derive(Clone, Copy)]
enum UnsafeCharacterId {
    Model(u16),
    Playable(u16),
    Runtime(u16),
    BossRuntime(u16),
}

impl UnsafeCharacterId {
    fn value(self) -> u16 {
        match self {
            Self::Model(id) | Self::Playable(id) | Self::Runtime(id) | Self::BossRuntime(id) => id,
        }
    }
}

fn unsafe_character_handle_table(lua: &Lua, query: Value) -> mlua::Result<Table> {
    let id = parse_unsafe_character_id(query)?;
    let value = id.value();
    let fields = lua.create_table()?;
    fields.set("name", format!("unsafe_{value}"))?;
    fields.set("known", false)?;
    fields.set("unsafe", true)?;
    fields.set("id", value)?;
    match id {
        UnsafeCharacterId::Model(id) => fields.set("model_id", id)?,
        UnsafeCharacterId::Playable(id) => fields.set("playable_id", id)?,
        UnsafeCharacterId::Runtime(id) => fields.set("runtime_id", id)?,
        UnsafeCharacterId::BossRuntime(id) => fields.set("boss_runtime_id", id)?,
    }
    custom_character_handle_table(lua, fields)
}

fn custom_character_handle_table(lua: &Lua, fields: Table) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let id = first_u16_field(
        &fields,
        &[
            "id",
            "model_id",
            "runtime_id",
            "playable_id",
            "boss_runtime_id",
        ],
    )?;
    let fallback_name = id
        .map(|id| format!("custom_{id}"))
        .unwrap_or_else(|| "custom_character".to_string());

    table.set("kind", "character")?;
    table.set(
        "known",
        fields.get::<Option<bool>>("known")?.unwrap_or(false),
    )?;
    table.set(
        "unsafe",
        fields.get::<Option<bool>>("unsafe")?.unwrap_or(false),
    )?;
    copy_optional_u16(&fields, &table, "id")?;
    copy_optional_u16(&fields, &table, "model_id")?;
    copy_optional_u16(&fields, &table, "playable_id")?;
    copy_optional_u16(&fields, &table, "runtime_id")?;
    copy_optional_u16(&fields, &table, "boss_runtime_id")?;
    copy_optional_u16(&fields, &table, "moveset_linkdata_entry")?;
    if table.get::<Option<u16>>("id")?.is_none() {
        if let Some(id) = id {
            table.set("id", id)?;
        }
    }
    let name = fields
        .get::<Option<String>>("name")?
        .or_else(|| fields.get::<Option<String>>("canonical").ok().flatten())
        .unwrap_or(fallback_name);
    let canonical = fields
        .get::<Option<String>>("canonical")?
        .unwrap_or_else(|| name.clone());
    table.set("name", name.as_str())?;
    table.set("canonical", canonical.as_str())?;
    table.set(
        "display_name",
        fields
            .get::<Option<String>>("display_name")?
            .unwrap_or_else(|| name.clone()),
    )?;
    table.set(
        "model_stem",
        fields
            .get::<Option<String>>("model_stem")?
            .unwrap_or_else(|| id.map(|id| format!("MPLC{id:03}")).unwrap_or_default()),
    )?;

    let metatable = lua.create_table()?;
    let methods: Table = lua.globals().get("__struct_api_methods")?;
    metatable.set(
        "__index",
        lua.create_function(move |_, (_this, key): (Table, String)| {
            methods.get::<Value>(key.as_str())
        })?,
    )?;
    table.set_metatable(Some(metatable));
    Ok(table)
}

fn first_u16_field(fields: &Table, keys: &[&str]) -> mlua::Result<Option<u16>> {
    for key in keys {
        if let Some(value) = fields.get::<Option<u16>>(*key)? {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn copy_optional_u16(source: &Table, target: &Table, key: &str) -> mlua::Result<()> {
    if let Some(value) = source.get::<Option<u16>>(key)? {
        target.set(key, value)?;
    }
    Ok(())
}

fn parse_unsafe_character_id(query: Value) -> mlua::Result<UnsafeCharacterId> {
    match query {
        Value::Integer(id) if (0..=u16::MAX as i64).contains(&id) => {
            Ok(UnsafeCharacterId::Model(id as u16))
        }
        Value::Table(table) => {
            if let Some(id) = table.get::<Option<u16>>("model_id")? {
                return Ok(UnsafeCharacterId::Model(id));
            }
            if let Some(id) = table.get::<Option<u16>>("id")? {
                return Ok(UnsafeCharacterId::Model(id));
            }
            if let Some(id) = table.get::<Option<u16>>("playable_id")? {
                return Ok(UnsafeCharacterId::Playable(id));
            }
            if let Some(id) = table.get::<Option<u16>>("runtime_id")? {
                return Ok(UnsafeCharacterId::Runtime(id));
            }
            if let Some(id) = table.get::<Option<u16>>("boss_runtime_id")? {
                return Ok(UnsafeCharacterId::BossRuntime(id));
            }
            Err(mlua::Error::external(
                "character.unsafe_find expects an id, model_id, playable_id, runtime_id, or boss_runtime_id",
            ))
        }
        _ => Err(mlua::Error::external(
            "character.unsafe_find expects a numeric id or id table",
        )),
    }
}

fn local_player_handle_table(lua: &Lua) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("kind", "local_player")?;
    table.set("id", -1)?;
    table.set("name", "local_player")?;
    table.set("canonical", "local_player")?;
    table.set("display_name", "Local Player")?;

    let metatable = lua.create_table()?;
    let methods: Table = lua.globals().get("__struct_api_methods")?;
    metatable.set(
        "__index",
        lua.create_function(move |_, (_this, key): (Table, String)| {
            methods.get::<Value>(key.as_str())
        })?,
    )?;
    table.set_metatable(Some(metatable));
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_returns_registered_module() {
        let lua = Lua::new();
        install_require_hook(&lua).expect("require hook");
        let module = lua.create_table().expect("table");
        module.set("value", 42).expect("value");
        register_module(&lua, "fx_director", module).expect("module");

        let value: i64 = lua
            .load(r#"local fx_director = require("fx_director"); return fx_director.value"#)
            .eval()
            .expect("eval");

        assert_eq!(value, 42);
    }

    #[test]
    fn required_plugin_can_extend_character_handles_once() {
        let lua = Lua::new();
        install_runtime(&lua).expect("runtime");
        let patcher = lua.create_table().expect("patcher");
        patcher
            .set(
                "__oppw4_on_import",
                lua.create_function(|lua, ()| {
                    let register: Function =
                        lua.globals().get("__oppw4_register_character_method")?;
                    let method =
                        lua.create_function(|_, (_this, slot, file): (Table, u16, String)| {
                            Ok(format!("{slot}:{file}"))
                        })?;
                    register.call::<()>(("skin_patcher", "replace_costume", method))
                })
                .expect("on import"),
            )
            .expect("hook");
        register_module(&lua, "skin_patcher", patcher).expect("module");

        let before: bool = lua
            .load(
                r#"
                local law = character.find("law")
                return law.replace_costume == nil
            "#,
            )
            .eval()
            .expect("before");
        assert!(before);

        let value: String = lua
            .load(
                r#"
                require("skin_patcher")
                local law = character.find("law")
                return law:replace_costume(3, "my_model.g1m")
            "#,
            )
            .eval()
            .expect("after");

        assert_eq!(value, "3:my_model.g1m");
    }

    #[test]
    fn character_find_accepts_known_ids() {
        let lua = Lua::new();
        install_runtime(&lua).expect("runtime");

        let value: String = lua
            .load(
                r#"
                local law_by_model = character.find(26)
                local law_by_playable = character.find(22)
                return law_by_model.name .. ":" .. law_by_playable.name
            "#,
            )
            .eval()
            .expect("eval");

        assert_eq!(value, "law:law");
    }

    #[test]
    fn character_unsafe_find_returns_unchecked_model_handle() {
        let lua = Lua::new();
        install_runtime(&lua).expect("runtime");

        let value: String = lua
            .load(
                r#"
                local custom = character.unsafe_find(730)
                return tostring(custom.known) .. ":" .. tostring(custom.unsafe) .. ":" .. custom.name .. ":" .. custom.model_id
            "#,
            )
            .eval()
            .expect("eval");

        assert_eq!(value, "false:true:unsafe_730:730");
    }

    #[test]
    fn character_unsafe_find_preserves_requested_id_kind() {
        let lua = Lua::new();
        install_runtime(&lua).expect("runtime");

        let value: String = lua
            .load(
                r#"
                local runtime = character.unsafe_find({ runtime_id = 730 })
                local playable = character.unsafe_find({ playable_id = 44 })
                return tostring(runtime.model_id) .. ":" .. runtime.runtime_id .. ":" .. tostring(playable.runtime_id) .. ":" .. playable.playable_id
            "#,
            )
            .eval()
            .expect("eval");

        assert_eq!(value, "nil:730:nil:44");
    }

    #[test]
    fn character_new_creates_custom_handles_with_plugin_methods() {
        let lua = Lua::new();
        install_runtime(&lua).expect("runtime");
        let patcher = lua.create_table().expect("patcher");
        patcher
            .set(
                "__oppw4_on_import",
                lua.create_function(|lua, ()| {
                    let register: Function =
                        lua.globals().get("__oppw4_register_character_method")?;
                    let method = lua.create_function(|_, (_this, effect): (Table, u16)| {
                        Ok(format!("fx:{effect}"))
                    })?;
                    register.call::<()>(("fx_director", "add_fx", method))
                })
                .expect("on import"),
            )
            .expect("hook");
        register_module(&lua, "fx_director", patcher).expect("module");

        let value: String = lua
            .load(
                r#"
                require("fx_director")
                local custom = character.new({
                    name = "my_custom_zoro",
                    runtime_id = 730,
                    model_stem = "CUSTOM_Zoro",
                })
                return custom.name .. ":" .. tostring(custom.model_id) .. ":" .. custom.runtime_id .. ":" .. custom:add_fx(2830)
            "#,
            )
            .eval()
            .expect("eval");

        assert_eq!(value, "my_custom_zoro:nil:730:fx:2830");
    }

    #[test]
    fn character_extension_method_conflicts_fail_loudly() {
        let lua = Lua::new();
        install_runtime(&lua).expect("runtime");

        let register: Function = lua
            .globals()
            .get("__oppw4_register_character_method")
            .expect("register");
        let first = lua.create_function(|_, ()| Ok(())).expect("first");
        let second = lua.create_function(|_, ()| Ok(())).expect("second");

        register
            .call::<()>(("skin_patcher", "replace_costume", first))
            .expect("first register");
        let error = register
            .call::<()>(("other_plugin", "replace_costume", second))
            .expect_err("conflict");

        assert!(error
            .to_string()
            .contains("already registered by skin_patcher"));
    }
}
