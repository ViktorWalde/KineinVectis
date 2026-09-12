//! Acoes que mudam COMO o projeto compila: rigor, sanitizers, paralelismo e
//! saida de firmware.
//!
//! POR QUE ESTE MODULO EXISTE (2026-09-04). Relato de uso do autor: *"falta
//! funcoes para exibir para o usuario selecionar como opcao. Tanto do C/C++
//! quanto do Cargo. Para deixar o compilador mais rigido ou para alguma
//! funcionalidade de sistemas embarcados ou de simulacao fisica/matematica"*.
//!
//! Ele estava certo: as dezoito acoes existentes respondiam "o que o projeto
//! TEM" — mais um executavel, mais uma dependencia, mais um include. Nenhuma
//! respondia "como o projeto COMPILA", que e' a pergunta de quem quer o
//! compilador mais rigido, quer sanitizer, quer `OpenMP` ou quer um `.hex` para
//! gravar numa placa.
//!
//! Sao dois vocabularios diferentes, e por isso arquivo diferente: o
//! `cmakelists.rs` acrescenta ENTIDADES ao projeto; este acrescenta REGIME de
//! compilacao.
//!
//! COMPILADOR: as flags aqui sao de `GCC` e `Clang`, que sao os compiladores que
//! esta IDE suporta (`DocsPublic/roadmaps/29`). MSVC usa outra grafia e nao esta no
//! escopo; a descricao de cada acao diz isso ao autor antes de ele aplicar.

use std::collections::BTreeMap;

use super::cmakelists::{
    CMAKELISTS, command_block, edit_plan, existing_target, insert_after_minimum_required,
    one_argument,
};
use super::error::ConfigActionError;
use super::plan::{ActionPlan, PlannedFile, read_required};
use super::{optional_param, required_param};

/// Avisos que valem a pena em qualquer projeto C/C++ serio.
///
/// A lista nao e' "tudo que existe": e' o conjunto que pega erro de verdade
/// sem afogar o autor em ruido. `-Wconversion` entra porque conversao
/// silenciosa de inteiro e' fonte classica de bug em embarcado, que e' um
/// alvo declarado deste projeto.
const STRICT_WARNINGS: [&str; 6] = [
    "-Wall",
    "-Wextra",
    "-Wpedantic",
    "-Wshadow",
    "-Wconversion",
    "-Wsign-conversion",
];

/// `target_compile_options` com o conjunto rigoroso de avisos.
///
/// `werror` e' opcional e desligado por padrao: transformar aviso em erro num
/// projeto que ja' tem avisos para o build inteiro, e a acao passaria de
/// "mais rigor" para "nada compila". Quem liga escolhe.
///
/// # Errors
/// `CMakeLists.txt` ausente, target inexistente ou parametro invalido.
pub(super) fn strict_warnings(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let (target, before) = existing_target(root, params)?;
    let mut flags: Vec<String> = STRICT_WARNINGS.iter().map(|f| (*f).to_owned()).collect();
    if liga(params, "werror") {
        flags.push("-Werror".to_owned());
    }
    let bloco = command_block(
        "target_compile_options",
        &format!("{target} PRIVATE"),
        &flags,
    );
    Ok(edit_plan(
        format!("Liga avisos rigorosos em {target} (GCC/Clang)"),
        before,
        &bloco,
    ))
}

/// Padrao de C++ fixado, exigido e sem extensoes do compilador.
///
/// As TRES linhas andam juntas de proposito. Sozinho, `CMAKE_CXX_STANDARD` e'
/// um pedido que o `CMake` pode ignorar em silencio se o compilador nao
/// suportar; `_REQUIRED ON` transforma isso em erro. E `_EXTENSIONS OFF` troca
/// `-std=gnu++20` por `-std=c++20`: sem isso o projeto compila com extensoes
/// da GNU e so' descobre no primeiro build com outro compilador.
///
/// # Errors
/// `CMakeLists.txt` ausente, ou o padrao ja' esta definido.
pub(super) fn cxx_standard(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let before = read_required(root, CMAKELISTS)?;
    if before.contains("CMAKE_CXX_STANDARD") {
        return Err(ConfigActionError::NotApplicable {
            reason: format!("{CMAKELISTS} ja define CMAKE_CXX_STANDARD"),
        });
    }
    let padrao = match optional_param(params, "standard") {
        Some(valor) => validar_padrao(&one_argument(valor, "standard")?)?,
        None => "20".to_owned(),
    };
    let bloco = format!(
        "set(CMAKE_CXX_STANDARD {padrao})\n\
         set(CMAKE_CXX_STANDARD_REQUIRED ON)\n\
         set(CMAKE_CXX_EXTENSIONS OFF)\n"
    );
    // NO TOPO, e nao no fim do arquivo. `CMAKE_CXX_STANDARD` e' lido quando o
    // TARGET E' CRIADO: escrito depois do `add_executable`, ele nao vira
    // propriedade de target nenhum e o build sai sem `-std=` — medido em
    // 2026-09-04, exercitando contra o `CMake` de verdade. A acao dizia
    // sucesso, o arquivo mudava, e a compilacao continuava igual.
    let after = insert_after_minimum_required(&before, &bloco);
    Ok(ActionPlan::edit(
        format!("Fixa o padrao C++{padrao}, exigido e sem extensoes GNU"),
        PlannedFile {
            path: CMAKELISTS.to_owned(),
            before: Some(before),
            after,
        },
    ))
}

/// `-fsanitize=...` no compilar E no linkar.
///
/// As duas metades sao obrigatorias e e' o erro mais comum de quem escreve
/// isso a mao: sanitizer so' nas flags de compilacao linka sem a runtime e
/// falha com "undefined reference to `__asan_...`".
///
/// # Errors
/// `CMakeLists.txt` ausente, target inexistente ou lista invalida.
pub(super) fn sanitizers(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let (target, before) = existing_target(root, params)?;
    // Sem passar pelo `one_argument`: ele exige UM token e a lista de
    // sanitizers e' separada por virgula por desenho do proprio compilador.
    // `validar_sanitizers` faz a checagem que importa aqui.
    let lista = match optional_param(params, "sanitizers") {
        Some(valor) => validar_sanitizers(valor)?,
        None => "address,undefined".to_owned(),
    };
    let compilar = command_block(
        "target_compile_options",
        &format!("{target} PRIVATE"),
        &[
            format!("-fsanitize={lista}"),
            "-fno-omit-frame-pointer".to_owned(),
            "-g".to_owned(),
        ],
    );
    let linkar = command_block(
        "target_link_options",
        &format!("{target} PRIVATE"),
        &[format!("-fsanitize={lista}")],
    );
    Ok(edit_plan(
        format!("Liga os sanitizers {lista} em {target}, no compilar e no linkar"),
        before,
        &format!("{compilar}{linkar}"),
    ))
}

/// `OpenMP` encontrado e linkado — o paralelismo de simulacao.
///
/// Duas linhas porque o `find_package` sozinho nao liga nada: e' o
/// `OpenMP::OpenMP_CXX` que traz `-fopenmp` para o compilar e para o linkar.
///
/// # Errors
/// `CMakeLists.txt` ausente ou target inexistente.
pub(super) fn openmp(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let (target, before) = existing_target(root, params)?;
    let bloco = format!(
        "find_package(OpenMP REQUIRED)\n\
         target_link_libraries({target} PRIVATE OpenMP::OpenMP_CXX)\n"
    );
    Ok(edit_plan(
        format!("Liga o OpenMP em {target} para paralelizar calculo"),
        before,
        &bloco,
    ))
}

/// `.hex` e `.bin` depois do build — o que se grava numa placa.
///
/// O ELF que o linker produz NAO e' o que a maioria dos gravadores aceita.
/// `objcopy` e' quem converte, e o `CMAKE_OBJCOPY` ja' aponta para o do
/// cross-compilador escolhido no kit — por isso a acao funciona sem
/// hard-code de `arm-none-eabi-objcopy`.
///
/// # Errors
/// `CMakeLists.txt` ausente ou target inexistente.
pub(super) fn hex_and_bin(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let (target, before) = existing_target(root, params)?;
    let bloco = format!(
        "add_custom_command(TARGET {target} POST_BUILD\n\
         \x20   COMMAND ${{CMAKE_OBJCOPY}} -O ihex $<TARGET_FILE:{target}> {target}.hex\n\
         \x20   COMMAND ${{CMAKE_OBJCOPY}} -O binary $<TARGET_FILE:{target}> {target}.bin\n\
         \x20   COMMENT \"Gerando {target}.hex e {target}.bin para gravacao\"\n\
         )\n"
    );
    Ok(edit_plan(
        format!("Gera {target}.hex e {target}.bin depois do build"),
        before,
        &bloco,
    ))
}

/// `.cargo/config.toml` apontando para uma placa.
///
/// Duas coisas de uma vez, e as duas sao o que faz `cargo run` funcionar num
/// microcontrolador: o alvo padrao do build e o RUNNER, que troca "executar
/// aqui" por "gravar e rodar la'". Sem o runner, `cargo run` tenta executar um
/// binario ARM na maquina do autor.
///
/// # Errors
/// Parametro ausente ou invalido, ou o arquivo ja' existe.
pub(super) fn cargo_embedded_target(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    const CARGO_CONFIG: &str = ".cargo/config.toml";

    let triple = validar_triple(&one_argument(required_param(params, "target")?, "target")?)?;
    let chip = one_argument(required_param(params, "chip")?, "chip")?;
    let caminho = root.join(".cargo").join("config.toml");
    if caminho.exists() {
        return Err(ConfigActionError::NotApplicable {
            reason: format!("{CARGO_CONFIG} ja existe; edite-o a mao para nao perder o conteudo"),
        });
    }
    let conteudo = format!(
        "[build]\n\
         target = \"{triple}\"\n\
         \n\
         [target.{triple}]\n\
         runner = \"probe-rs run --chip {chip}\"\n"
    );
    Ok(ActionPlan::edit(
        format!("Cria {CARGO_CONFIG} para compilar e gravar em {chip} ({triple})"),
        PlannedFile {
            path: CARGO_CONFIG.to_owned(),
            before: None,
            after: conteudo,
        },
    ))
}

/// `true` quando o parametro booleano esta ligado.
fn liga(params: &BTreeMap<String, String>, nome: &str) -> bool {
    optional_param(params, nome).is_some_and(|valor| {
        matches!(
            valor.trim().to_ascii_uppercase().as_str(),
            "ON" | "TRUE" | "YES" | "1"
        )
    })
}

/// Padrao de C++ que o `CMake` reconhece.
fn validar_padrao(raw: &str) -> Result<String, ConfigActionError> {
    const ACEITOS: [&str; 6] = ["11", "14", "17", "20", "23", "26"];
    if ACEITOS.contains(&raw) {
        return Ok(raw.to_owned());
    }
    Err(ConfigActionError::InvalidParam {
        name: "standard",
        reason: format!("padrao C++ invalido: use um de {}", ACEITOS.join(", ")),
    })
}

/// Sanitizers que `GCC` e `Clang` aceitam juntos.
///
/// `address` e `thread` sao mutuamente exclusivos, e o compilador so' reclama
/// disso no fim do build. Recusar aqui poupa uma compilacao inteira.
fn validar_sanitizers(raw: &str) -> Result<String, ConfigActionError> {
    const ACEITOS: [&str; 4] = ["address", "undefined", "thread", "leak"];
    let pedidos: Vec<&str> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if pedidos.is_empty() {
        return Err(ConfigActionError::InvalidParam {
            name: "sanitizers",
            reason: "informe ao menos um sanitizer".to_owned(),
        });
    }
    for pedido in &pedidos {
        if !ACEITOS.contains(pedido) {
            return Err(ConfigActionError::InvalidParam {
                name: "sanitizers",
                reason: format!(
                    "sanitizer desconhecido `{pedido}`: use {}",
                    ACEITOS.join(", ")
                ),
            });
        }
    }
    if pedidos.contains(&"address") && pedidos.contains(&"thread") {
        return Err(ConfigActionError::InvalidParam {
            name: "sanitizers",
            reason: "address e thread nao podem ser ligados juntos".to_owned(),
        });
    }
    Ok(pedidos.join(","))
}

/// Triple de alvo do Rust, no formato que o `cargo` aceita.
fn validar_triple(raw: &str) -> Result<String, ConfigActionError> {
    let valido = !raw.is_empty()
        && raw.len() <= 64
        && raw
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.');
    if valido {
        Ok(raw.to_owned())
    } else {
        Err(ConfigActionError::InvalidParam {
            name: "target",
            reason: "triple invalido (ex.: thumbv7em-none-eabihf)".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn temp_root(nome: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-configaction-rigor")
            .join(format!("{}-{nome}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.24)\nproject(demo CXX)\nadd_executable(demo main.cpp)\n",
        )
        .unwrap();
        dir
    }

    fn parametros(pares: &[(&str, &str)]) -> BTreeMap<String, String> {
        pares
            .iter()
            .map(|(chave, valor)| ((*chave).to_owned(), (*valor).to_owned()))
            .collect()
    }

    fn escrito(plano: &ActionPlan) -> String {
        plano
            .files
            .first()
            .map(|f| f.after.clone())
            .unwrap_or_default()
    }

    /// A falha que a exercitacao contra o `CMake` REAL achou em 2026-09-04: o
    /// bloco saia no FIM do arquivo, depois do `add_executable`, e o build
    /// continuava sem `-std=` nenhum. A acao dizia sucesso e nao mudava nada.
    #[test]
    fn o_padrao_cpp_entra_antes_do_target_existir() {
        let root = temp_root("padrao");
        let plano = cxx_standard(&root, &parametros(&[])).unwrap();
        let texto = escrito(&plano);

        let posicao_padrao = texto.find("CMAKE_CXX_STANDARD").unwrap();
        let posicao_target = texto.find("add_executable").unwrap();
        assert!(
            posicao_padrao < posicao_target,
            "o padrao saiu DEPOIS do target; ele nao teria efeito:\n{texto}"
        );
        assert!(texto.contains("CMAKE_CXX_STANDARD_REQUIRED ON"));
        assert!(texto.contains("CMAKE_CXX_EXTENSIONS OFF"));
    }

    #[test]
    fn padrao_invalido_e_recusado_com_a_lista_do_que_vale() {
        let root = temp_root("padrao-invalido");
        let erro = cxx_standard(&root, &parametros(&[("standard", "19")])).unwrap_err();
        assert!(format!("{erro}").contains("20"), "{erro}");
    }

    #[test]
    fn werror_so_entra_quando_pedido() {
        let root = temp_root("werror");
        let sem = escrito(&strict_warnings(&root, &parametros(&[("target", "demo")])).unwrap());
        assert!(sem.contains("-Wall") && !sem.contains("-Werror"));

        let com = escrito(
            &strict_warnings(&root, &parametros(&[("target", "demo"), ("werror", "ON")])).unwrap(),
        );
        assert!(com.contains("-Werror"));
    }

    /// Sanitizer so' nas flags de compilacao linka sem a runtime e falha com
    /// "undefined reference to `__asan_...`". As duas metades andam juntas.
    #[test]
    fn sanitizer_entra_no_compilar_e_no_linkar() {
        let root = temp_root("sanitizer");
        let texto = escrito(&sanitizers(&root, &parametros(&[("target", "demo")])).unwrap());
        assert!(texto.contains("target_compile_options"));
        assert!(texto.contains("target_link_options"));
        assert_eq!(texto.matches("-fsanitize=address,undefined").count(), 2);
    }

    /// `address` e `thread` sao incompativeis, e o compilador so' reclama no
    /// fim do build. Recusar aqui poupa uma compilacao inteira.
    #[test]
    fn address_e_thread_juntos_sao_recusados_antes_de_compilar() {
        let root = temp_root("sanitizer-conflito");
        let erro = sanitizers(
            &root,
            &parametros(&[("target", "demo"), ("sanitizers", "address,thread")]),
        )
        .unwrap_err();
        assert!(
            format!("{erro}").contains("nao podem ser ligados juntos"),
            "{erro}"
        );

        let desconhecido = sanitizers(
            &root,
            &parametros(&[("target", "demo"), ("sanitizers", "memoria")]),
        )
        .unwrap_err();
        assert!(
            format!("{desconhecido}").contains("memoria"),
            "{desconhecido}"
        );
    }

    #[test]
    fn openmp_encontra_e_linka() {
        let root = temp_root("openmp");
        let texto = escrito(&openmp(&root, &parametros(&[("target", "demo")])).unwrap());
        assert!(texto.contains("find_package(OpenMP REQUIRED)"));
        assert!(texto.contains("OpenMP::OpenMP_CXX"));
    }

    #[test]
    fn hex_e_bin_usam_o_objcopy_do_kit_e_nao_um_nome_fixo() {
        let root = temp_root("hexbin");
        let texto = escrito(&hex_and_bin(&root, &parametros(&[("target", "demo")])).unwrap());
        assert!(
            texto.contains("${CMAKE_OBJCOPY}"),
            "hard-code de objcopy quebraria o cross-compilador do kit"
        );
        assert!(texto.contains("-O ihex") && texto.contains("-O binary"));
    }

    /// Sem o runner, `cargo run` tenta executar um binario ARM na maquina do
    /// autor. O alvo sozinho nao basta.
    #[test]
    fn o_config_do_cargo_traz_alvo_e_runner() {
        let root = temp_root("cargo-embarcado");
        let plano = cargo_embedded_target(
            &root,
            &parametros(&[
                ("target", "thumbv7em-none-eabihf"),
                ("chip", "STM32F401RETx"),
            ]),
        )
        .unwrap();
        let texto = escrito(&plano);
        assert!(texto.contains("target = \"thumbv7em-none-eabihf\""));
        assert!(texto.contains("runner = \"probe-rs run --chip STM32F401RETx\""));
        assert_eq!(plano.files[0].path, ".cargo/config.toml");
        assert!(
            plano.files[0].before.is_none(),
            "arquivo novo nao tem `before`"
        );
    }

    #[test]
    fn config_do_cargo_existente_nao_e_sobrescrito() {
        let root = temp_root("cargo-existente");
        std::fs::create_dir_all(root.join(".cargo")).unwrap();
        std::fs::write(root.join(".cargo/config.toml"), "[build]\n").unwrap();
        let erro = cargo_embedded_target(
            &root,
            &parametros(&[("target", "thumbv7em-none-eabihf"), ("chip", "X")]),
        )
        .unwrap_err();
        assert!(format!("{erro}").contains("ja existe"), "{erro}");
    }
}
