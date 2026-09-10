//! Simulacoes salvas em `.kinein/simulacoes/`, uma por arquivo.
//!
//! Segue o precedente do `runconfig.rs`: `schemaVersion` no arquivo, e arquivo
//! invalido ou de schema desconhecido tratado como AUSENTE em vez de quebrar o
//! fluxo. Quem abre a IDE com um `.kinein/` corrompido perde a simulacao, nao a
//! sessao.
//!
//! ## Uma por arquivo, e nao um catalogo
//!
//! Um arquivo unico com todas dentro poria dois trabalhos sem relacao no mesmo
//! diff, e todo conflito de merge seria entre simulacoes que nao se conhecem —
//! metade do problema do `.slx` do Simulink, cujo fabricante manda registrar a
//! extensao como binaria e vende a ferramenta de merge a parte.
//!
//! ## O RESULTADO nao entra
//!
//! O arquivo guarda o que o autor MONTOU: conceito, formula, ligacao, valores,
//! metodo, passo. Nao guarda a trilha, nem os numeros, nem o grafico. E' a licao
//! precisa do `.ipynb` (`arquitetura/34` §9): ele e' texto, e' diffavel, e mesmo
//! assim falha porque mistura o autoral com a saida da maquina.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kinein_protocol::SimSaved;
use serde::{Deserialize, Serialize};

/// Versao do schema escrita por este core.
const SCHEMA_VERSION: u32 = 1;

/// Representacao em disco de uma simulacao.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Arquivo {
    schema_version: u32,
    #[serde(flatten)]
    simulacao: SimSaved,
}

/// A pasta das simulacoes de um workspace.
#[must_use]
pub fn pasta(root: &Path) -> PathBuf {
    root.join(".kinein").join("simulacoes")
}

/// Transforma o nome que o autor deu num nome de arquivo seguro.
///
/// Nao e' cosmetica: sem isto, um nome com `../` escreveria fora do workspace, e
/// um com `/` criaria diretorio. O nome ORIGINAL fica dentro do arquivo, entao
/// nada se perde na traducao — o que muda e' so' o caminho no disco.
#[must_use]
pub fn nome_de_arquivo(nome: &str) -> String {
    let limpo: String = nome
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let cortado = limpo.trim_matches('-').to_string();
    if cortado.is_empty() {
        "simulacao".to_string()
    } else {
        cortado
    }
}

/// Le todas as simulacoes salvas, em ordem de nome.
///
/// Arquivo invalido e' PULADO, nao propaga erro: uma simulacao corrompida nao
/// pode esconder as outras.
#[must_use]
pub fn listar(root: &Path) -> Vec<SimSaved> {
    let Ok(entradas) = fs::read_dir(pasta(root)) else {
        return Vec::new();
    };
    let mut caminhos: Vec<PathBuf> = entradas
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    caminhos.sort();
    caminhos
        .iter()
        .filter_map(|caminho| {
            let corpo = fs::read_to_string(caminho).ok()?;
            let arquivo: Arquivo = serde_json::from_str(&corpo).ok()?;
            (arquivo.schema_version == SCHEMA_VERSION).then_some(arquivo.simulacao)
        })
        .collect()
}

/// Grava uma simulacao. Nome vazio e' recusado.
pub fn salvar(root: &Path, simulacao: &SimSaved) -> Result<Vec<SimSaved>, String> {
    if simulacao.name.trim().is_empty() {
        return Err("a simulacao precisa de um nome".to_owned());
    }
    let destino = pasta(root);
    fs::create_dir_all(&destino)
        .map_err(|erro| format!("falha criando {}: {erro}", destino.display()))?;
    let arquivo = Arquivo {
        schema_version: SCHEMA_VERSION,
        simulacao: simulacao.clone(),
    };
    let corpo = serde_json::to_string_pretty(&arquivo)
        .map_err(|erro| format!("falha serializando a simulacao: {erro}"))?;
    let caminho = destino.join(format!("{}.json", nome_de_arquivo(&simulacao.name)));
    fs::write(&caminho, corpo).map_err(|erro| format!("falha escrevendo a simulacao: {erro}"))?;
    Ok(listar(root))
}

/// Apaga uma simulacao pelo nome.
pub fn esquecer(root: &Path, nome: &str) -> Result<Vec<SimSaved>, String> {
    let caminho = pasta(root).join(format!("{}.json", nome_de_arquivo(nome)));
    match fs::remove_file(&caminho) {
        Ok(()) => Ok(listar(root)),
        Err(erro) if erro.kind() == std::io::ErrorKind::NotFound => Ok(listar(root)),
        Err(erro) => Err(format!("falha removendo a simulacao: {erro}")),
    }
}
