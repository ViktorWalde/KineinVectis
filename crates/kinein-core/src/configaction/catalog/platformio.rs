//! As acoes do `PlatformIO`.

use kinein_protocol::{ConfigActionEffect, ConfigActionRisk, ConfigActionScope};

use super::ActionDefinition;

pub(super) static ACTIONS: [ActionDefinition; 1] = [ActionDefinition {
    id: "platformio.compiledb",
    title: "Gerar compile_commands.json",
    description: "Roda pio run -t compiledb: o clangd passa a ver as flags do PlatformIO.",
    scope: ConfigActionScope::Platformio,
    category: "PlatformIO",
    risk: ConfigActionRisk::Low,
    affects: &["compile_commands.json"],
    effect: ConfigActionEffect::Job,
    params: &[],
    docs: &[(
        "officialDoc",
        "PlatformIO: compilation database",
        "platformio.compiledb",
    )],
}];
