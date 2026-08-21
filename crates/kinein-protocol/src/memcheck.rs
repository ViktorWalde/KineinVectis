//! Analise dinamica de memoria (`memcheck.run`, L2 fatia 6).
//!
//! Fecha a triade de analise do L2. A estatica le o codigo; esta ve o que ele
//! FAZ ao rodar. Vazamento, escrita fora do bloco e leitura de valor nao
//! inicializado nao aparecem em nenhuma analise estatica deste projeto.
//!
//! A analise roda os TESTES do projeto sob o Valgrind — nao um binario
//! escolhido a dedo. E o uso que rende: teste ja exercita os caminhos, e o
//! `CTest` ja sabe o comando exato de cada um.

use serde::{Deserialize, Serialize};

/// Parametros de `memcheck.run`.
///
/// Vazio e com `deny_unknown_fields`: o alvo sao os testes do workspace
/// aberto, e nao ha `buildSystem` a escolher porque so o caminho `CMake` tem
/// analise dinamica hoje. Campo que nasceria ignorado e peso morto de
/// contrato (licao do `assistant_terminal_width`).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemcheckRunParams {}

#[cfg(test)]
mod tests {
    use super::MemcheckRunParams;

    #[test]
    fn params_recusam_campo_desconhecido() {
        assert!(serde_json::from_str::<MemcheckRunParams>("{}").is_ok());
        assert!(serde_json::from_str::<MemcheckRunParams>(r#"{"buildSystem":"cmake"}"#).is_err());
    }
}
