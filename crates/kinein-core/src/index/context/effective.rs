//! O contexto EFETIVO de um arquivo, com a origem de cada valor (0.4.0a,
//! `roadmaps/52` §14.1, protocolo `0.173.0`).
//!
//! Derivado a cada pergunta, nunca guardado. A precedencia e' a do 52 §4.3,
//! "o valor vem do dono": as flags reais (a unidade da CDB ou do file-api)
//! vencem a escolha do kit, e o kit vence o que o cache do `CMake` registra.
//! Sem fonte, o campo fica ausente: nada de padrao suposto (a regra da 16.7b).

use kinein_protocol::{CompileUnit, ContextOrigin, EffectiveContext, KitRef, SourcedValue};

/// O que o kit que vale diz, ja' resolvido pelo dono (`toolchain`).
#[derive(Debug, Clone, Default)]
pub struct KitFacts {
    /// O preset do kit; vazio e' o kit padrao.
    pub preset: String,
    /// O compilador que a pessoa FIXOU para a linguagem do arquivo.
    pub pinned_compiler: Option<String>,
    /// O compilador que o cache do `CMake` do projeto registra, sem nada fixado.
    pub cache_compiler: Option<String>,
    /// O `targetTriple` do kit.
    pub target_triple: Option<String>,
    /// O `sysroot` do kit.
    pub sysroot: Option<String>,
}

fn sourced(value: &str, origin: ContextOrigin) -> SourcedValue {
    SourcedValue {
        value: value.to_owned(),
        origin,
    }
}

/// O que a unidade de compilacao sabe: compilador, padrao, alvo e sysroot.
///
/// `sysroot` e' o que o file-api do `CMake` declara ao lado da unidade (a CDB
/// o traz nas flags).
pub(super) fn from_unit(
    unit: &CompileUnit,
    origin: ContextOrigin,
    sysroot: Option<&str>,
) -> EffectiveContext {
    EffectiveContext {
        // O file-api sem compilador conhecido escreve "(CXX do kit)": e' uma
        // frase, nao um valor.
        compiler: (!unit.compiler.starts_with('(')).then(|| sourced(&unit.compiler, origin)),
        standard: unit.standard.as_deref().map(|value| sourced(value, origin)),
        target: flag_value(&unit.arguments, &["--target", "-target"])
            .map(|value| sourced(&value, origin)),
        sysroot: sysroot
            .map(str::to_owned)
            .or_else(|| flag_value(&unit.arguments, &["--sysroot"]))
            .map(|value| sourced(&value, origin)),
        flags_origin: Some(origin),
        kit: None,
    }
}

/// `--flag=valor`, ou `--flag valor`.
fn flag_value(arguments: &[String], names: &[&str]) -> Option<String> {
    let mut iter = arguments.iter();
    let mut result = None;
    while let Some(argument) = iter.next() {
        for name in names {
            if let Some(value) = argument
                .strip_prefix(name)
                .and_then(|rest| rest.strip_prefix('='))
            {
                result = Some(value.to_owned());
                break;
            }
            if argument == name {
                result = iter.next().cloned();
                break;
            }
        }
    }
    result
}

/// Completa com o kit o que a unidade nao disse, e diz qual kit vale.
pub fn complete_with_kit(effective: &mut EffectiveContext, kit: &KitFacts) {
    if effective.compiler.is_none() {
        effective.compiler = kit
            .pinned_compiler
            .as_deref()
            .map(|value| sourced(value, ContextOrigin::Kit))
            .or_else(|| {
                kit.cache_compiler
                    .as_deref()
                    .map(|value| sourced(value, ContextOrigin::CmakeCache))
            });
    }
    if effective.target.is_none() {
        effective.target = kit
            .target_triple
            .as_deref()
            .map(|value| sourced(value, ContextOrigin::Kit));
    }
    if effective.sysroot.is_none() {
        effective.sysroot = kit
            .sysroot
            .as_deref()
            .map(|value| sourced(value, ContextOrigin::Kit));
    }
    effective.kit = Some(KitRef {
        preset: kit.preset.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(compiler: &str, arguments: &[&str]) -> CompileUnit {
        CompileUnit {
            compiler: compiler.to_owned(),
            directory: "/p/build".to_owned(),
            standard: Some("gnu++20".to_owned()),
            includes: Vec::new(),
            defines: Vec::new(),
            output: None,
            arguments: arguments.iter().map(|a| (*a).to_owned()).collect(),
        }
    }

    fn kit() -> KitFacts {
        KitFacts {
            preset: "nucleo".to_owned(),
            pinned_compiler: Some("/opt/arm/bin/arm-none-eabi-g++".to_owned()),
            cache_compiler: Some("/usr/bin/c++".to_owned()),
            target_triple: Some("thumbv7em-none-eabihf".to_owned()),
            sysroot: Some("/opt/sysroot-do-kit".to_owned()),
        }
    }

    /// As flags reais vencem o kit: compilador, padrao, alvo e sysroot da CDB,
    /// nas duas formas (`--flag=valor` e `--flag valor`).
    #[test]
    fn the_unit_flags_win_over_the_kit() {
        let mut effective = from_unit(
            &unit(
                "/usr/bin/clang++",
                &[
                    "/usr/bin/clang++",
                    "--target=aarch64-linux-gnu",
                    "--sysroot",
                    "/srv/raiz",
                ],
            ),
            ContextOrigin::Cdb,
            None,
        );
        complete_with_kit(&mut effective, &kit());
        assert_eq!(
            effective.compiler,
            Some(sourced("/usr/bin/clang++", ContextOrigin::Cdb))
        );
        assert_eq!(
            effective.standard,
            Some(sourced("gnu++20", ContextOrigin::Cdb))
        );
        assert_eq!(
            effective.target,
            Some(sourced("aarch64-linux-gnu", ContextOrigin::Cdb))
        );
        assert_eq!(
            effective.sysroot,
            Some(sourced("/srv/raiz", ContextOrigin::Cdb))
        );
        assert_eq!(effective.flags_origin, Some(ContextOrigin::Cdb));
        assert_eq!(
            effective.kit,
            Some(KitRef {
                preset: "nucleo".to_owned()
            })
        );
    }

    /// O que a unidade nao diz vem do kit; o sysroot do file-api vale como o
    /// da unidade; a frase "(CXX do kit)" nao vira compilador.
    #[test]
    fn what_the_unit_does_not_say_comes_from_the_kit() {
        let mut effective = from_unit(
            &unit("(CXX do kit)", &["-O2"]),
            ContextOrigin::CmakeFileApi,
            Some("/opt/sysroot-do-file-api"),
        );
        complete_with_kit(&mut effective, &kit());
        assert_eq!(
            effective.compiler,
            Some(sourced(
                "/opt/arm/bin/arm-none-eabi-g++",
                ContextOrigin::Kit
            ))
        );
        assert_eq!(
            effective.target,
            Some(sourced("thumbv7em-none-eabihf", ContextOrigin::Kit))
        );
        assert_eq!(
            effective.sysroot,
            Some(sourced(
                "/opt/sysroot-do-file-api",
                ContextOrigin::CmakeFileApi
            ))
        );
    }

    /// Sem unidade e sem nada fixado: o compilador do cache do `CMake`; sem
    /// cache nem kit, tudo ausente, e o kit padrao dito como vazio.
    #[test]
    fn without_a_unit_the_cache_then_nothing() {
        let mut effective = EffectiveContext::default();
        complete_with_kit(
            &mut effective,
            &KitFacts {
                pinned_compiler: None,
                ..kit()
            },
        );
        assert_eq!(
            effective.compiler,
            Some(sourced("/usr/bin/c++", ContextOrigin::CmakeCache))
        );

        let mut empty = EffectiveContext::default();
        complete_with_kit(&mut empty, &KitFacts::default());
        assert_eq!(empty.compiler, None);
        assert_eq!(empty.target, None);
        assert_eq!(empty.sysroot, None);
        assert_eq!(empty.flags_origin, None);
        assert_eq!(
            empty.kit,
            Some(KitRef {
                preset: String::new()
            })
        );
    }
}
