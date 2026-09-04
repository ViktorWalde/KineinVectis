//! Dobrar documentos num mapa de campos — sem servidor, sem rede.
//!
//! # Por que isto e' um arquivo proprio
//!
//! O `mongo.rs` ao lado FALA com o servidor: conecta, lista colecoes, pede a
//! amostra. Este aqui nao sabe que existe um servidor: recebe documentos e
//! devolve campos. Sao duas responsabilidades, e a segunda e' a unica que da'
//! para exercitar exaustivamente sem subir um banco.
//!
//! # A conta acontece AQUI, e nao no servidor (decisao do autor, 2026-09-04)
//!
//! A alternativa era um `$objectToArray` + `$unwind` + `$group` no servidor:
//! rede minima, RAM da IDE quase zero — em troca de tres estagios de agregacao
//! rodando no banco DO AUTOR, com teto de 100 MB no `$group` e um `$unwind`
//! que multiplica linhas em documento aninhado.
//!
//! A regra que a IDE ja segue com o `PostgreSQL` decidiu: o banco do usuario e'
//! para o trabalho dele. A IDE le'; ela nao pede que ele calcule.
//!
//! # O teto de RAM e' explicito, e ele APARECE (decisao do autor, 2026-09-04)
//!
//! O documento e' descartado assim que e' dobrado, entao o pico nao cresce com
//! o tamanho da colecao. O que PODE crescer sem limite e' o mapa de caminhos:
//! uma colecao que usa identificador como NOME de campo (`chave_1`,
//! `chave_2`, ...) gera um caminho novo por documento. Os tres tetos abaixo
//! fecham isso, e quando um deles morde a tela diz — em vez de mostrar uma
//! arvore que parece completa.

use std::collections::{BTreeMap, BTreeSet};

use kinein_protocol::MongoField;
use mongodb::bson::{Bson, Document};

/// Quantos caminhos distintos o mapa guarda.
///
/// Duas mil linhas ja' e' mais do que qualquer humano varre com o olho; alem
/// disto a arvore deixa de ser leitura e vira despejo.
pub const MAX_FIELDS: usize = 2_000;

/// Ate' onde a leitura desce em documento aninhado.
///
/// Oito niveis cobrem qualquer modelagem que alguem defenda numa revisao. Mais
/// que isso, o caminho `a.b.c.d.e.f.g.h.i` ja' nao cabe na tela nem na cabeca.
pub const MAX_DEPTH: u16 = 8;

/// Quantos elementos de um array sao inspecionados.
///
/// Array de subdocumento e' o coracao da modelagem do `MongoDB`, entao ignora-lo
/// deixaria a arvore quase inutil. Mas um array de dez mil elementos nao ensina
/// dez mil vezes mais que um de dez: os primeiros ja' mostram a forma.
pub const MAX_ARRAY_ELEMENTS: usize = 10;

/// O que se sabe de um caminho.
#[derive(Debug, Default)]
struct Stats {
    /// Tipos BSON vistos, ordenados pelo `BTreeSet`.
    types: BTreeSet<&'static str>,
    /// Em quantos DOCUMENTOS o caminho apareceu — nunca quantas vezes.
    documents: u32,
    depth: u16,
}

/// Acumula a amostra. Um documento entra, e sai da memoria.
#[derive(Debug, Default)]
pub struct Inference {
    fields: BTreeMap<String, Stats>,
    documents: u32,
    /// Primeiro teto que mordeu, se algum.
    truncated: Option<String>,
}

impl Inference {
    /// Dobra mais um documento no mapa.
    pub fn add(&mut self, document: &Document) {
        self.documents += 1;
        // OS CAMINHOS DESTE DOCUMENTO, uma vez cada. Sem este conjunto, um
        // array com tres subdocumentos contaria `tags.nome` tres vezes e a
        // presenca passaria de 100% — a tela afirmaria que o campo aparece em
        // mais documentos do que existem.
        let mut vistos: BTreeMap<String, (&'static str, u16)> = BTreeMap::new();
        self.walk(document, "", 0, &mut vistos);
        for (caminho, (tipo, profundidade)) in vistos {
            let entrada = self.fields.entry(caminho).or_insert_with(|| Stats {
                depth: profundidade,
                ..Stats::default()
            });
            entrada.types.insert(tipo);
            entrada.documents += 1;
        }
    }

    /// Percorre um documento, anotando os caminhos vistos.
    fn walk(
        &mut self,
        document: &Document,
        prefixo: &str,
        profundidade: u16,
        vistos: &mut BTreeMap<String, (&'static str, u16)>,
    ) {
        for (chave, valor) in document {
            let caminho = if prefixo.is_empty() {
                chave.clone()
            } else {
                format!("{prefixo}.{chave}")
            };
            // O TETO DE CAMINHOS morde na ENTRADA de um caminho novo, e o que
            // ja' esta' no mapa continua sendo contado: cortar pela metade um
            // campo ja' conhecido daria uma presenca menor que a verdadeira.
            if !self.fields.contains_key(&caminho)
                && !vistos.contains_key(&caminho)
                && self.fields.len() + vistos.len() >= MAX_FIELDS
            {
                self.truncated
                    .get_or_insert_with(|| format!("parei em {MAX_FIELDS} campos distintos"));
                continue;
            }
            vistos.insert(caminho.clone(), (type_name(valor), profundidade));
            self.descend(valor, &caminho, profundidade, vistos);
        }
    }

    /// Desce em documento e em array de documento, respeitando os tetos.
    fn descend(
        &mut self,
        valor: &Bson,
        caminho: &str,
        profundidade: u16,
        vistos: &mut BTreeMap<String, (&'static str, u16)>,
    ) {
        if profundidade + 1 > MAX_DEPTH {
            if matches!(valor, Bson::Document(_) | Bson::Array(_)) {
                self.truncated
                    .get_or_insert_with(|| format!("parei em {MAX_DEPTH} níveis de aninhamento"));
            }
            return;
        }
        match valor {
            Bson::Document(interno) => self.walk(interno, caminho, profundidade + 1, vistos),
            Bson::Array(itens) => {
                for item in itens.iter().take(MAX_ARRAY_ELEMENTS) {
                    if let Bson::Document(interno) = item {
                        // O MESMO CAMINHO do array: `tags` e `tags.nome`. Um
                        // indice no caminho (`tags.0.nome`) descreveria a
                        // amostra, nao a forma — e a forma e' a pergunta.
                        self.walk(interno, caminho, profundidade + 1, vistos);
                    }
                }
            }
            _ => {}
        }
    }

    /// Quantos documentos entraram.
    #[must_use]
    pub const fn documents(&self) -> u32 {
        self.documents
    }

    /// A frase do teto que mordeu, ou vazio.
    #[must_use]
    pub fn truncated(&self) -> String {
        self.truncated.clone().unwrap_or_default()
    }

    /// Os campos, ordenados pelo caminho, com a presenca calculada.
    #[must_use]
    pub fn into_fields(self) -> Vec<MongoField> {
        let total = f64::from(self.documents.max(1));
        self.fields
            .into_iter()
            .map(|(path, stats)| MongoField {
                depth: stats.depth,
                types: stats.types.into_iter().map(str::to_owned).collect(),
                presence: Some(f64::from(stats.documents) / total),
                required: false,
                path,
            })
            .collect()
    }
}

/// O nome do tipo BSON, na grafia que o `MongoDB` usa em `$type`.
///
/// A grafia importa: e' a que o autor vai digitar num filtro, e traduzir para
/// nomes proprios da IDE obrigaria a traduzir de volta na cabeca.
const fn type_name(valor: &Bson) -> &'static str {
    match valor {
        Bson::Double(_) => "double",
        Bson::String(_) => "string",
        Bson::Array(_) => "array",
        Bson::Document(_) => "object",
        Bson::Boolean(_) => "bool",
        Bson::Null => "null",
        Bson::RegularExpression(_) => "regex",
        Bson::JavaScriptCode(_) | Bson::JavaScriptCodeWithScope(_) => "javascript",
        Bson::Int32(_) => "int",
        Bson::Int64(_) => "long",
        Bson::Timestamp(_) => "timestamp",
        Bson::Binary(_) => "binData",
        Bson::ObjectId(_) => "objectId",
        Bson::DateTime(_) => "date",
        Bson::Symbol(_) => "symbol",
        Bson::Decimal128(_) => "decimal",
        Bson::Undefined => "undefined",
        Bson::MaxKey => "maxKey",
        Bson::MinKey => "minKey",
        Bson::DbPointer(_) => "dbPointer",
    }
}

#[cfg(test)]
mod tests {
    use mongodb::bson::doc;

    use super::*;

    fn campos(documentos: &[Document]) -> Vec<MongoField> {
        let mut inferencia = Inference::default();
        for documento in documentos {
            inferencia.add(documento);
        }
        inferencia.into_fields()
    }

    fn achar<'a>(campos: &'a [MongoField], caminho: &str) -> &'a MongoField {
        campos
            .iter()
            .find(|campo| campo.path == caminho)
            .unwrap_or_else(|| panic!("caminho `{caminho}` ausente"))
    }

    /// CAMPO OPCIONAL E' O PONTO. Uma tabela diria que ele existe sempre.
    #[test]
    fn presenca_conta_documentos_e_nao_ocorrencias() {
        let campos = campos(&[
            doc! { "nome": "a", "extra": 1 },
            doc! { "nome": "b" },
            doc! { "nome": "c" },
            doc! { "nome": "d" },
        ]);
        assert!((achar(&campos, "nome").presence.unwrap() - 1.0).abs() < 1e-9);
        assert!((achar(&campos, "extra").presence.unwrap() - 0.25).abs() < 1e-9);
    }

    /// TIPO VARIAVEL E' O OUTRO PONTO. Uma coluna tem um tipo so'.
    #[test]
    fn o_mesmo_campo_com_dois_tipos_mostra_os_dois() {
        let campos = campos(&[doc! { "firmware": "v1" }, doc! { "firmware": 2_i32 }]);
        assert_eq!(achar(&campos, "firmware").types, vec!["int", "string"]);
    }

    #[test]
    fn documento_aninhado_vira_caminho_com_ponto() {
        let campos = campos(&[doc! { "meta": { "placa": "esp32", "canal": 1_i32 } }]);
        assert_eq!(achar(&campos, "meta").types, vec!["object"]);
        assert_eq!(achar(&campos, "meta.placa").types, vec!["string"]);
        assert_eq!(achar(&campos, "meta.placa").depth, 1);
    }

    /// O array de subdocumento e' a modelagem tipica do Mongo; ignora-lo
    /// deixaria a arvore quase vazia num projeto real.
    #[test]
    fn array_de_documento_e_aberto_no_mesmo_caminho() {
        let campos = campos(&[doc! { "tags": [ { "nome": "a" }, { "nome": "b" } ] }]);
        assert_eq!(achar(&campos, "tags").types, vec!["array"]);
        assert_eq!(achar(&campos, "tags.nome").types, vec!["string"]);
    }

    /// A ARMADILHA: tres elementos com o mesmo campo dariam 300% de presenca.
    #[test]
    fn array_nao_infla_a_presenca_acima_de_um() {
        let campos = campos(&[doc! { "tags": [ { "n": 1_i32 }, { "n": 2_i32 }, { "n": 3_i32 } ] }]);
        let presenca = achar(&campos, "tags.n").presence.unwrap();
        assert!(
            (presenca - 1.0).abs() < 1e-9,
            "presenca inflada pelo array: {presenca}"
        );
    }

    /// O TETO DE CAMPOS morde, e DIZ que mordeu.
    #[test]
    fn chave_dinamica_bate_no_teto_e_avisa() {
        let mut inferencia = Inference::default();
        for indice in 0..(MAX_FIELDS + 500) {
            inferencia.add(
                &doc! { format!("chave_{indice}"): i64::try_from(indice).unwrap_or_default() },
            );
        }
        let aviso = inferencia.truncated();
        assert!(aviso.contains("campos distintos"), "sem aviso: {aviso}");
        let campos = inferencia.into_fields();
        assert!(
            campos.len() <= MAX_FIELDS,
            "o teto nao segurou: {} campos",
            campos.len()
        );
    }

    /// Campo que JA' esta' no mapa continua contando depois do teto: cortar um
    /// caminho conhecido daria uma presenca menor que a verdadeira.
    #[test]
    fn campo_conhecido_continua_contando_depois_do_teto() {
        let mut inferencia = Inference::default();
        for indice in 0..(MAX_FIELDS + 100) {
            inferencia.add(&doc! { "estavel": 1_i32, format!("chave_{indice}"): 1_i32 });
        }
        let campos = inferencia.into_fields();
        let estavel = achar(&campos, "estavel").presence.unwrap();
        assert!(
            (estavel - 1.0).abs() < 1e-9,
            "campo estavel perdeu presenca: {estavel}"
        );
    }

    /// O teto de profundidade morde, e tambem avisa.
    #[test]
    fn aninhamento_profundo_para_e_avisa() {
        let mut documento = doc! { "folha": 1_i32 };
        for _ in 0..(MAX_DEPTH + 3) {
            documento = doc! { "n": documento };
        }
        let mut inferencia = Inference::default();
        inferencia.add(&documento);
        assert!(inferencia.truncated().contains("aninhamento"));
        let campos = inferencia.into_fields();
        assert!(campos.iter().all(|campo| campo.depth <= MAX_DEPTH));
    }

    /// Amostra vazia nao divide por zero nem entrega campo nenhum.
    #[test]
    fn amostra_vazia_nao_quebra() {
        let inferencia = Inference::default();
        assert_eq!(inferencia.documents(), 0);
        assert!(inferencia.into_fields().is_empty());
    }
}
