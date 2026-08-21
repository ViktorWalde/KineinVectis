//! Analise DINAMICA de memoria via Valgrind/Memcheck (L2 fatia 6).
//!
//! Fecha a triade de analise do L2: estatica (Cppcheck + clang-tidy),
//! seguranca (cargo-audit) e dinamica (aqui). As tres respondem perguntas
//! diferentes — a estatica le o codigo, a dinamica ve o que ele FAZ ao rodar.
//! Vazamento, escrita fora do bloco e leitura de memoria nao inicializada nao
//! aparecem em nenhuma analise estatica deste projeto; so aqui.
//!
//! Nada aqui instrumenta nada: o Valgrind e a ferramenta madura, e o que este
//! modulo faz e enumerar o que rodar, rodar, e normalizar o resultado.

use std::{
    error::Error,
    fmt,
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::{BuildDiagnostic, BuildDiagnosticSeverity};

use crate::process::{self, ProcessError};

/// Erro ao rodar a analise dinamica.
#[derive(Debug)]
pub enum MemcheckError {
    /// A ferramenta nao pode ser iniciada (provavelmente ausente).
    Spawn {
        /// Comando que falhou.
        command: String,
        /// Erro de IO subjacente.
        source: std::io::Error,
    },
    /// Falha de IO durante a execucao.
    Io(std::io::Error),
    /// O projeto nao foi configurado, entao nao ha o que rodar.
    SemBuild,
    /// O projeto foi configurado mas nao declara nenhum teste.
    SemTestes,
}

impl fmt::Display for MemcheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { command, source } => {
                write!(formatter, "falha ao iniciar '{command}': {source}")
            }
            Self::Io(error) => write!(formatter, "falha de IO durante a analise: {error}"),
            // Erros ACIONAVEIS: dizem o gesto que falta, nao so o sintoma.
            Self::SemBuild => write!(
                formatter,
                "o projeto ainda nao foi configurado — rode o build uma vez \
                 antes da analise dinamica"
            ),
            Self::SemTestes => write!(
                formatter,
                "o projeto nao declara testes no ctest; a analise dinamica roda \
                 os testes sob o Valgrind e nao tem o que executar"
            ),
        }
    }
}

impl Error for MemcheckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } | Self::Io(source) => Some(source),
            Self::SemBuild | Self::SemTestes => None,
        }
    }
}

/// Um teste a rodar sob o Valgrind, como o `CTest` o declara.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TesteExecutavel {
    /// Nome do teste, para aparecer na saida do job.
    pub nome: String,
    /// Comando completo: executavel + argumentos.
    pub comando: Vec<String>,
}

/// Resultado de uma analise dinamica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemcheckReport {
    /// Achados, ja no formato do funil de diagnostico.
    pub findings: Vec<BuildDiagnostic>,
    /// Quantos executaveis foram analisados.
    pub testes: u64,
}

/// Roda os testes do projeto sob o Valgrind e normaliza os achados.
pub fn run_memcheck(
    root: &Path,
    extra_args: &[String],
    cancel: &Arc<AtomicBool>,
    on_output: &mut dyn FnMut(&str),
) -> Result<MemcheckReport, MemcheckError> {
    let build_dir = root.join(".kinein").join("build");
    if !build_dir.join("CMakeCache.txt").is_file() {
        return Err(MemcheckError::SemBuild);
    }

    let testes = listar_testes(&build_dir, cancel)?;
    if testes.is_empty() {
        return Err(MemcheckError::SemTestes);
    }

    let mut findings = Vec::new();
    for teste in &testes {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        on_output(&format!("== {} ==", teste.nome));

        let mut command = Command::new("valgrind");
        command
            .args(valgrind_args(extra_args))
            .args(&teste.comando)
            .current_dir(root);

        let mut bruto = String::new();
        let mut sink = |_stream: &'static str, linha: String| {
            bruto.push_str(&linha);
            bruto.push('\n');
            on_output(&linha);
        };
        process::stream_command_lines_cancelable(command, cancel, &mut sink).map_err(|error| {
            match error {
                ProcessError::Spawn(source) => MemcheckError::Spawn {
                    command: "valgrind".to_owned(),
                    source,
                },
                ProcessError::Wait(source) => MemcheckError::Io(source),
            }
        })?;
        findings.extend(parse_valgrind(&bruto, root));
    }

    Ok(MemcheckReport {
        findings,
        testes: testes.len() as u64,
    })
}

/// Argumentos do Valgrind + extras da configuracao da integracao.
///
/// `--error-exitcode=0` pela MESMA razao do Cppcheck: quem decide severidade e
/// a UI, pelo diagnostico, nao o codigo de saida do processo. Um achado nao
/// pode derrubar o job — se derrubasse, o proximo teste nem rodaria.
#[must_use]
pub fn valgrind_args(extra_args: &[String]) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "--tool=memcheck".to_owned(),
        "--leak-check=full".to_owned(),
        // Sem isto o Valgrind so diz QUE ha valor nao inicializado, nao ONDE
        // ele nasceu — e "onde nasceu" e a unica parte acionavel.
        "--track-origins=yes".to_owned(),
        "--error-exitcode=0".to_owned(),
    ];
    args.extend(extra_args.iter().cloned());
    args
}

/// Enumera os testes pelo proprio `CTest`, em vez de adivinhar executaveis.
///
/// `--show-only=json-v1` devolve o comando EXATO de cada teste, com argumentos
/// ja resolvidos pelo `CMake`. Varrer o diretorio de build atras de binarios
/// daria a lista errada: pega utilitario, pula teste que precisa de argumento,
/// e ignora quem o projeto excluiu.
fn listar_testes(
    build_dir: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<TesteExecutavel>, MemcheckError> {
    let mut command = Command::new("ctest");
    command
        .arg("--test-dir")
        .arg(build_dir)
        .arg("--show-only=json-v1");

    let mut saida = String::new();
    let mut sink = |stream: &'static str, linha: String| {
        if stream == "stdout" {
            saida.push_str(&linha);
            saida.push('\n');
        }
    };
    process::stream_command_lines_cancelable(command, cancel, &mut sink).map_err(|error| {
        match error {
            ProcessError::Spawn(source) => MemcheckError::Spawn {
                command: "ctest".to_owned(),
                source,
            },
            ProcessError::Wait(source) => MemcheckError::Io(source),
        }
    })?;

    Ok(parse_ctest_json(&saida))
}

/// Parse da lista de testes do `ctest --show-only=json-v1`.
#[must_use]
pub fn parse_ctest_json(raw: &str) -> Vec<TesteExecutavel> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw.trim()) else {
        return Vec::new();
    };
    let Some(testes) = value.get("tests").and_then(serde_json::Value::as_array) else {
        return Vec::new();
    };
    testes
        .iter()
        .filter_map(|teste| {
            let nome = teste.get("name")?.as_str()?.to_owned();
            let comando: Vec<String> = teste
                .get("command")?
                .as_array()?
                .iter()
                .filter_map(|parte| parte.as_str().map(ToOwned::to_owned))
                .collect();
            // Teste sem comando nao e executavel: o CTest usa isso para
            // fixtures e placeholders, e roda-los seria erro nosso.
            if comando.is_empty() {
                return None;
            }
            Some(TesteExecutavel { nome, comando })
        })
        .collect()
}

/// Linhas de banner e de resumo do Valgrind, que NAO sao achados.
///
/// Sem esta lista o "HEAP SUMMARY:" viraria um diagnostico, e a aba Problemas
/// encheria de linha decorativa. A distincao principal e outra — achado nao e
/// indentado e resumo e —, mas estes escapam por nao serem indentados.
const RUIDO: &[&str] = &[
    "Memcheck,",
    "Copyright ",
    "Using Valgrind",
    "Command:",
    "Parent PID:",
    "HEAP SUMMARY:",
    "LEAK SUMMARY:",
    "ERROR SUMMARY:",
    "All heap blocks were freed",
    "For lists of detected",
    "For counts of detected",
    "Use --track-origins",
    "Rerun with --leak-check",
];

/// Parse da saida de texto do Valgrind (medido na versao 3.24.0).
///
/// Forma real, ja com o prefixo `==PID==` de cada linha:
///
/// ```text
/// ==123== Invalid write of size 4
/// ==123==    at 0x109160: main (vaza.c:5)
/// ==123==  Address 0x... is 7 bytes inside a block of size 10 alloc'd
/// ==123==    at 0x4844818: malloc (vg_replace_malloc.c:446)
/// ==123==    by 0x10914A: main (vaza.c:4)
/// ```
///
/// A regra que separa achado de decoracao e a INDENTACAO: o cabecalho do erro
/// nao e indentado, e quadro de pilha e detalhe sao. `--leak-check=full` usa
/// o mesmo desenho para vazamento ("N bytes in M blocks are definitely lost").
#[must_use]
pub fn parse_valgrind(raw: &str, root: &Path) -> Vec<BuildDiagnostic> {
    let mut findings: Vec<BuildDiagnostic> = Vec::new();
    let mut aberto: Option<usize> = None;

    for linha in raw.lines() {
        let Some(conteudo) = linha.split_once("== ").map(|(_, resto)| resto) else {
            // Linha `==PID==` vazia fecha o achado em aberto.
            if linha.trim_end().ends_with("==") {
                aberto = None;
            }
            continue;
        };
        if conteudo.trim().is_empty() {
            aberto = None;
            continue;
        }

        let indentado = conteudo.starts_with(' ');
        if indentado {
            // Pode ser um quadro de pilha. So o PRIMEIRO quadro util posiciona
            // o achado — os seguintes sao a cadeia de chamada, e apontar para o
            // meio dela levaria o clique ao lugar errado.
            let Some(indice) = aberto else { continue };
            if findings[indice].file.is_some() {
                continue;
            }
            if let Some((arquivo, numero)) = quadro(conteudo, root) {
                findings[indice].file = Some(arquivo);
                findings[indice].line = Some(numero);
            }
            continue;
        }

        if RUIDO.iter().any(|ruido| conteudo.starts_with(ruido)) {
            aberto = None;
            continue;
        }
        findings.push(BuildDiagnostic {
            severity: severidade(conteudo),
            message: conteudo.trim().to_owned(),
            file: None,
            line: None,
            column: None,
        });
        aberto = Some(findings.len() - 1);
    }

    findings
}

/// Severidade do achado pelo texto do cabecalho.
///
/// Vazamento e AVISO e acesso invalido e ERRO, e a diferenca importa: ler fora
/// do bloco corrompe o programa AGORA; vazar memoria e uma divida que talvez
/// nunca cobre. Tratar os dois igual afogaria o erro no meio dos avisos.
fn severidade(cabecalho: &str) -> BuildDiagnosticSeverity {
    let baixo = cabecalho.to_ascii_lowercase();
    if baixo.contains("lost") || baixo.contains("still reachable") || baixo.contains("suppressed") {
        BuildDiagnosticSeverity::Warning
    } else {
        BuildDiagnosticSeverity::Error
    }
}

/// Extrai `(arquivo, linha)` de um quadro de pilha `at`/`by`.
///
/// Descarta quadro de dentro do proprio Valgrind (`vg_replace_malloc.c`) e o
/// que fica fora da raiz do workspace: a pilha atravessa a libc e o
/// interpretador, e mandar o usuario para o `malloc.c` da glibc nao ajuda.
fn quadro(conteudo: &str, root: &Path) -> Option<(String, u64)> {
    let corte = conteudo.trim_start();
    if !corte.starts_with("at 0x") && !corte.starts_with("by 0x") {
        return None;
    }
    let abre = conteudo.rfind('(')?;
    let fecha = conteudo.rfind(')')?;
    if fecha <= abre {
        return None;
    }
    let dentro = &conteudo[abre + 1..fecha];
    // `(in /usr/lib/libfoo.so)` nao tem linha: nao e posicao util.
    let (arquivo, numero) = dentro.rsplit_once(':')?;
    let numero: u64 = numero.parse().ok()?;
    if arquivo.ends_with("vg_replace_malloc.c") {
        return None;
    }

    let caminho = Path::new(arquivo);
    if caminho.is_absolute() {
        if !caminho.starts_with(root) {
            return None;
        }
        let relativo = caminho.strip_prefix(root).ok()?;
        return Some((relativo.to_string_lossy().into_owned(), numero));
    }
    Some((arquivo.to_owned(), numero))
}

#[cfg(test)]
mod tests {
    use super::{parse_ctest_json, parse_valgrind, valgrind_args};
    use kinein_protocol::BuildDiagnosticSeverity;
    use std::path::Path;

    // Recorte FIEL do valgrind 3.24.0, medido em 2026-08-21 sobre um programa
    // com estouro de bloco e vazamento.
    const SAIDA: &str = "\
==209589== Memcheck, a memory error detector\n\
==209589== Copyright (C) 2002-2024, and GNU GPL'd, by Julian Seward et al.\n\
==209589== Using Valgrind-3.24.0 and LibVEX; rerun with -h for copyright info\n\
==209589== Command: /ws/vaza\n\
==209589== \n\
==209589== Invalid write of size 4\n\
==209589==    at 0x109160: main (/ws/src/vaza.c:5)\n\
==209589==  Address 0x4a6e047 is 7 bytes inside a block of size 10 alloc'd\n\
==209589==    at 0x4844818: malloc (vg_replace_malloc.c:446)\n\
==209589==    by 0x10914A: main (/ws/src/vaza.c:4)\n\
==209589== \n\
==209589== HEAP SUMMARY:\n\
==209589==     in use at exit: 14 bytes in 2 blocks\n\
==209589== \n\
==209589== 4 bytes in 1 blocks are definitely lost in loss record 1 of 2\n\
==209589==    at 0x4844818: malloc (vg_replace_malloc.c:446)\n\
==209589==    by 0x109170: main (/ws/src/vaza.c:6)\n\
==209589== \n\
==209589== LEAK SUMMARY:\n\
==209589==    definitely lost: 14 bytes in 2 blocks\n\
==209589== \n\
==209589== ERROR SUMMARY: 4 errors from 4 contexts (suppressed: 0 from 0)\n";

    #[test]
    fn parse_separa_achado_de_decoracao_e_posiciona_no_primeiro_quadro_util() {
        let achados = parse_valgrind(SAIDA, Path::new("/ws"));

        assert_eq!(
            achados.len(),
            2,
            "so o acesso invalido e o vazamento: banner, HEAP/LEAK/ERROR \
             SUMMARY e detalhe indentado NAO sao achados"
        );

        let invalido = &achados[0];
        assert_eq!(invalido.message, "Invalid write of size 4");
        assert_eq!(invalido.severity, BuildDiagnosticSeverity::Error);
        // O primeiro quadro util, ja RELATIVO a raiz.
        assert_eq!(invalido.file.as_deref(), Some("src/vaza.c"));
        assert_eq!(invalido.line, Some(5));

        let vazamento = &achados[1];
        assert!(vazamento.message.contains("definitely lost"));
        assert_eq!(
            vazamento.severity,
            BuildDiagnosticSeverity::Warning,
            "vazar e divida; ler fora do bloco corrompe agora — severidades \
             diferentes, ou o erro afoga no meio dos avisos"
        );
        // O quadro do malloc do proprio Valgrind e PULADO: a posicao util e
        // a linha do projeto que alocou.
        assert_eq!(vazamento.file.as_deref(), Some("src/vaza.c"));
        assert_eq!(vazamento.line, Some(6));
    }

    #[test]
    fn quadro_fora_da_raiz_nao_posiciona() {
        // A pilha atravessa a libc; mandar o usuario para o malloc.c da glibc
        // nao ajuda em nada.
        let saida = "\
==1== Invalid read of size 8\n\
==1==    at 0x4844818: memcpy (/usr/lib/glibc/memcpy.c:99)\n\
==1==    by 0x109170: main (/ws/src/a.c:12)\n";
        let achados = parse_valgrind(saida, Path::new("/ws"));
        assert_eq!(achados.len(), 1);
        assert_eq!(achados[0].file.as_deref(), Some("src/a.c"));
        assert_eq!(achados[0].line, Some(12));
    }

    #[test]
    fn achado_sem_quadro_util_ainda_aparece() {
        // Sem posicao continua sendo um achado: some-lo seria esconder um
        // problema real so porque nao sabemos apontar o dedo.
        let saida = "\
==1== Conditional jump depends on uninitialised value(s)\n\
==1==    at 0x4844818: ??? (in /usr/lib/libfoo.so)\n";
        let achados = parse_valgrind(saida, Path::new("/ws"));
        assert_eq!(achados.len(), 1);
        assert!(achados[0].file.is_none());
        assert_eq!(achados[0].severity, BuildDiagnosticSeverity::Error);
    }

    #[test]
    fn ctest_json_da_o_comando_exato_e_pula_teste_sem_comando() {
        let raw = r#"{"tests": [
            {"name": "tst_a", "command": ["/ws/build/tst_a", "--gtest_color=no"]},
            {"name": "fixture", "command": []},
            {"name": "tst_b", "command": ["/ws/build/tst_b"]}
        ]}"#;
        let testes = parse_ctest_json(raw);
        assert_eq!(testes.len(), 2, "teste sem comando nao e executavel");
        assert_eq!(testes[0].nome, "tst_a");
        assert_eq!(
            testes[0].comando,
            vec!["/ws/build/tst_a", "--gtest_color=no"]
        );
        assert_eq!(testes[1].nome, "tst_b");

        // JSON invalido nao vira lista inventada.
        assert!(parse_ctest_json("nao e json").is_empty());
        assert!(parse_ctest_json("{}").is_empty());
    }

    #[test]
    fn args_pedem_origem_e_nao_derrubam_o_job_por_achado() {
        let args = valgrind_args(&[]);
        assert!(args.contains(&"--leak-check=full".to_owned()));
        assert!(
            args.contains(&"--track-origins=yes".to_owned()),
            "sem origem, valor nao inicializado nao e acionavel"
        );
        assert!(
            args.contains(&"--error-exitcode=0".to_owned()),
            "achado nao pode derrubar o job: o proximo teste nem rodaria"
        );

        // Extras da config entram DEPOIS, entao o usuario sobrepoe.
        let extras = vec!["--show-leak-kinds=all".to_owned()];
        let com_extras = valgrind_args(&extras);
        assert_eq!(com_extras.last().unwrap(), "--show-leak-kinds=all");
    }
}
