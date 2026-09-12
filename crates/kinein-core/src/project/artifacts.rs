//! O que o ultimo build DEIXOU: ELF, imagens, mapa, a receita de gravacao.
//!
//! Pilar 0 do `roadmaps/42`. O `dap::resolve_program` acha "o unico ELF" por
//! convencao; o `build.size` acha "o unico .ld". O modelo lista TODOS os
//! candidatos que ve, do mais novo ao mais velho, e diz onde: a decisao de
//! qual usar e' de quem grava/depura, com o usuario confirmando. Onde olhar
//! vem de cada framework — o `build/` do ESP-IDF e do Zephyr, o `.pio/build/
//! <env>/` do `PlatformIO`, o `target/<triple>/` do cargo, o `.kinein/build` da
//! propria IDE.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use kinein_protocol::ProjectArtifacts;

/// Quantos candidatos de cada tipo o modelo carrega. Cinco e' "os recentes".
const MAXIMO_POR_TIPO: usize = 5;

/// Ate' onde descer DENTRO de uma pasta de build.
const PROFUNDIDADE_BUILD: usize = 3;

/// Os artefatos sob `root`.
#[must_use]
pub fn artifacts(root: &Path) -> ProjectArtifacts {
    let mut art = ProjectArtifacts::default();
    let mut vistos: Vec<(SystemTime, PathBuf, Tipo)> = Vec::new();

    for build in pastas_de_build(root) {
        varrer(&build, 0, &mut vistos);
    }
    vistos.sort_by_key(|v| std::cmp::Reverse(v.0));
    for (_, caminho, tipo) in vistos {
        let texto = caminho.display().to_string();
        let lista = match tipo {
            Tipo::Elf => &mut art.elf,
            Tipo::Bin => &mut art.bin,
            Tipo::Hex => &mut art.hex,
            Tipo::Uf2 => &mut art.uf2,
            Tipo::Map => &mut art.map,
            Tipo::FlasherArgs => {
                art.flasher_args.get_or_insert(texto);
                continue;
            }
            Tipo::PartitionTable => {
                art.partition_table.get_or_insert(texto);
                continue;
            }
        };
        if lista.len() < MAXIMO_POR_TIPO {
            lista.push(texto);
        }
    }

    // O que mora na FONTE, nao no build: tabela de particoes em CSV, memory.x,
    // linker scripts do projeto.
    if art.partition_table.is_none() {
        let csv = root.join("partitions.csv");
        if csv.is_file() {
            art.partition_table = Some(csv.display().to_string());
        }
    }
    let memory_x = root.join("memory.x");
    if memory_x.is_file() {
        art.memory_x = Some(memory_x.display().to_string());
    }
    art.linker_scripts = linker_scripts(root);

    // ESP-IDF: o que estava so' LOCALIZADO passa a ser LIDO (segunda fatia do
    // pilar 0). A receita resolve caminhos contra a pasta do proprio JSON.
    art.flash_recipe = art.flasher_args.as_deref().and_then(|caminho| {
        let caminho = Path::new(caminho);
        let json = std::fs::read_to_string(caminho).ok()?;
        super::esp::flash_recipe(&json, caminho.parent().unwrap_or(root))
    });
    art.partitions = art
        .partition_table
        .as_deref()
        .filter(|p| {
            Path::new(p)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("csv"))
        })
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|csv| super::esp::partition_table(&csv, super::esp::TABLE_OFFSET_PADRAO));
    art
}

#[derive(Debug, Clone, Copy)]
enum Tipo {
    Elf,
    Bin,
    Hex,
    Uf2,
    Map,
    FlasherArgs,
    PartitionTable,
}

/// As pastas onde cada framework escreve. So' as que existem.
fn pastas_de_build(root: &Path) -> Vec<PathBuf> {
    let mut pastas = vec![
        root.join(".kinein").join("build"),
        root.join("build"),
        root.join("cmake-build-debug"),
        root.join("cmake-build-release"),
    ];
    // PlatformIO: .pio/build/<env>
    if let Ok(envs) = std::fs::read_dir(root.join(".pio").join("build")) {
        pastas.extend(envs.filter_map(Result::ok).map(|e| e.path()));
    }
    // cargo: target/<triple>/{debug,release} — so' os triples (bare metal);
    // target/debug e' o host e ja' e' do dap::resolve_program.
    if let Ok(alvos) = std::fs::read_dir(root.join("target")) {
        for alvo in alvos.filter_map(Result::ok) {
            let nome = alvo.file_name().to_string_lossy().into_owned();
            if nome.contains('-') {
                pastas.push(alvo.path().join("debug"));
                pastas.push(alvo.path().join("release"));
            }
        }
    }
    pastas.into_iter().filter(|p| p.is_dir()).collect()
}

fn varrer(dir: &Path, nivel: usize, vistos: &mut Vec<(SystemTime, PathBuf, Tipo)>) {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };
    for entrada in entradas.filter_map(Result::ok) {
        let caminho = entrada.path();
        let Ok(meta) = entrada.metadata() else {
            continue;
        };
        if meta.is_dir() {
            // `deps/`, `build/` do cargo e `CMakeFiles/` sao intermediarios.
            let nome = entrada.file_name().to_string_lossy().into_owned();
            if nivel < PROFUNDIDADE_BUILD
                && !matches!(
                    nome.as_str(),
                    "deps" | "build" | "CMakeFiles" | "incremental" | ".fingerprint"
                )
            {
                varrer(&caminho, nivel + 1, vistos);
            }
            continue;
        }
        if let Some(tipo) = tipo_de(&caminho, meta.len()) {
            let quando = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            vistos.push((quando, caminho, tipo));
        }
    }
}

/// Pela extensao — e, sem extensao, pelos 4 bytes magicos do ELF (o cargo
/// gera o binario do alvo sem extensao).
fn tipo_de(caminho: &Path, tamanho: u64) -> Option<Tipo> {
    let nome = caminho.file_name()?.to_string_lossy().into_owned();
    if nome == "flasher_args.json" {
        return Some(Tipo::FlasherArgs);
    }
    if nome == "partition-table.bin" || nome == "partitions.csv" {
        return Some(Tipo::PartitionTable);
    }
    match caminho.extension().and_then(|e| e.to_str()) {
        Some("elf") => Some(Tipo::Elf),
        Some("bin") => Some(Tipo::Bin),
        Some("hex") => Some(Tipo::Hex),
        Some("uf2") => Some(Tipo::Uf2),
        Some("map") => Some(Tipo::Map),
        None if tamanho >= 4 && e_elf(caminho) => Some(Tipo::Elf),
        _ => None,
    }
}

fn e_elf(caminho: &Path) -> bool {
    use std::io::Read;
    let Ok(mut arquivo) = std::fs::File::open(caminho) else {
        return false;
    };
    let mut magico = [0u8; 4];
    arquivo.read_exact(&mut magico).is_ok() && magico == [0x7f, b'E', b'L', b'F']
}

/// Os `.ld` da FONTE (raiz e um nivel abaixo), fora das pastas de build — o
/// mesmo criterio do `build.size`, que so' usa quando ha' exatamente um; o
/// modelo lista todos e deixa a escolha para quem mede.
fn linker_scripts(root: &Path) -> Vec<String> {
    let mut achados = Vec::new();
    let mut pastas = vec![root.to_path_buf()];
    if let Ok(entradas) = std::fs::read_dir(root) {
        for entrada in entradas.filter_map(Result::ok) {
            let nome = entrada.file_name().to_string_lossy().into_owned();
            if entrada.path().is_dir()
                && !matches!(
                    nome.as_str(),
                    "build" | ".kinein" | "target" | ".pio" | ".git"
                )
            {
                pastas.push(entrada.path());
            }
        }
    }
    for pasta in pastas {
        if let Ok(entradas) = std::fs::read_dir(&pasta) {
            for entrada in entradas.filter_map(Result::ok) {
                let caminho = entrada.path();
                if caminho.extension().is_some_and(|e| e == "ld") && caminho.is_file() {
                    achados.push(caminho.display().to_string());
                }
            }
        }
    }
    achados.sort();
    achados
}
