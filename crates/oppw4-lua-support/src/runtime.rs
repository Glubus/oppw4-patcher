use mlua::{Function, Lua, Table};

use crate::LuaMod;

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
    install_import_alias(&lua).map_err(LuaRunError::Lua)?;
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

pub fn install_import_alias(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    let require: Function = globals.get("require")?;
    globals.set("import", require)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_alias_uses_require() {
        let lua = Lua::new();
        install_import_alias(&lua).expect("alias");
        let module = lua.create_table().expect("table");
        module.set("value", 42).expect("value");
        register_module(&lua, "aura", module).expect("module");

        let value: i64 = lua
            .load(r#"local aura = import("aura"); return aura.value"#)
            .eval()
            .expect("eval");

        assert_eq!(value, 42);
    }
}
