//! As acoes do ESP-IDF: projetos `CMake` do framework (52 §14.41).

use kinein_protocol::{ConfigActionEffect, ConfigActionRisk, ConfigActionScope};

use super::ActionDefinition;

pub(super) static ACTIONS: [ActionDefinition; 1] = [ActionDefinition {
    id: "espidf.clangdFreeRtos",
    title: "Clangd: manter os headers do FreeRTOS",
    description: "Acrescenta ao .clangd um IgnoreHeader para freertos/: o include-cleaner deixa \
                  de sugerir trocar freertos/FreeRTOS.h, o que quebra o build.",
    scope: ConfigActionScope::Cmake,
    category: "ESP-IDF",
    risk: ConfigActionRisk::Low,
    affects: &[".clangd"],
    effect: ConfigActionEffect::Edit,
    params: &[],
    docs: &[(
        "officialDoc",
        "clangd: configuração, Diagnostics.Includes.IgnoreHeader",
        "clangd.config.includes",
    )],
}];
