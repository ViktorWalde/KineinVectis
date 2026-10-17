//! O kit como toolchain file (0.4.1a, `roadmaps/52` §14.7): os MESMOS valores
//! que o configure da IDE passa pela linha ([`Toolchain::cmake_definitions`]),
//! para o terminal e o CI (`cmake --toolchain <arquivo>`).
//!
//! A IDE nao le o arquivo. Medido em 2026-10-17 (`CMake` 4.2.3): o `set()` de
//! um toolchain file vence o `-D` da linha — lido, ele deixaria o kit mudado
//! depois perder em silencio. O kit continua a fonte; o arquivo e' a foto dele.

use std::fmt::Write as _;

use super::Toolchain;

/// O arquivo exportado: onde, relativo a raiz, e o que ele diz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedToolchain {
    /// `.kinein/toolchains/<kit>.cmake`.
    pub path: String,
    /// O conteudo, sem data: exportar de novo o mesmo kit da' o mesmo texto.
    pub contents: String,
}

impl Toolchain {
    /// O toolchain file do kit `kit`, ou o motivo de nao haver um.
    ///
    /// # Errors
    ///
    /// O kit ja' usa um toolchain file (do preset ou dele), ou nao e' cross.
    pub fn export(&self, kit: &str) -> Result<ExportedToolchain, String> {
        if let Some(file) = self
            .preset_toolchain_file
            .as_deref()
            .or(self.kit_toolchain_file.as_deref())
        {
            return Err(format!("o kit já usa um toolchain file: {file}"));
        }
        let definitions = self.cmake_definitions();
        if !definitions
            .iter()
            .any(|(name, _)| *name == "CMAKE_SYSTEM_NAME")
        {
            return Err(
                "o kit não é cross: sem um alvo (triple) que o CMake reconheça, o configure \
                 da IDE não passa nada que um toolchain file guarde"
                    .to_owned(),
            );
        }
        let path = format!(".kinein/toolchains/{}.cmake", file_stem(kit));
        let label = if kit.is_empty() { "padrão" } else { kit };
        let mut contents = format!(
            "# O kit \"{label}\" da Kinein Vectis: os mesmos valores do configure da IDE.\n\
             # A IDE não lê este arquivo; o kit continua a fonte. Uso:\n\
             #   cmake --toolchain {path} -S . -B <build>\n\n"
        );
        for (name, value) in definitions {
            // Escrever numa `String` nao falha.
            let _ = writeln!(contents, "set({name} {})", quoted(&value));
        }
        Ok(ExportedToolchain { path, contents })
    }
}

/// O nome do arquivo: o do kit, so' com o que um nome de arquivo aceita em
/// toda plataforma; o kit padrao vira `padrao`.
fn file_stem(kit: &str) -> String {
    if kit.is_empty() {
        return "padrao".to_owned();
    }
    kit.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Um argumento entre aspas do `CMake`: `\`, `"` e `$` escapados (o `${` seria
/// expandido).
fn quoted(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_quotes_survive_odd_input() {
        assert_eq!(file_stem(""), "padrao");
        assert_eq!(file_stem("arm cortex/m4"), "arm_cortex_m4");
        assert_eq!(file_stem("rpi-4.release_1"), "rpi-4.release_1");
        assert_eq!(quoted("/opt/a b"), "\"/opt/a b\"");
        assert_eq!(quoted(r#"C:\x"${y}"#), r#""C:\\x\"\${y}""#);
    }
}
