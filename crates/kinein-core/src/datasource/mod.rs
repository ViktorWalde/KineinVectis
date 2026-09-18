//! Fontes de dados: as conexoes do autor a bancos relacionais e temporais.
//!
//! Este dominio nasceu em 2026-09-04 como a etapa 26 do
//! `DocsPublic/roadmaps/35-ambiente-cpp-embarcados-simulacao.md`, e a primeira coisa
//! que ele faz e' cumprir a regra que o §7.3 daquele documento escreveu:
//!
//! > *"Nenhuma linha de conexao a banco entra antes dessa pergunta ter dono."*
//!
//! A pergunta era onde mora a senha. A resposta esta' registrada em
//! `DocsPublic/seguranca/40-cofre-de-credencial.md` (autor, 2026-09-04): **a IDE
//! guarda o PERFIL, que nao e' segredo, e nunca a senha.**
//!
//! # O que este dominio e' e o que ele NAO e'
//!
//! ```text
//! E'        o catalogo de perfis: nome, host, porta, banco, usuario, e a
//!           POLITICA de onde buscar a senha na hora de conectar
//! NAO E'    o cliente de banco. Nao ha' driver aqui, nao ha' conexao aberta,
//!           nao ha' SQL. Isso e' a fatia seguinte, e ela depende desta.
//! ```
//!
//! A separacao nao e' burocracia: o perfil e' PERSISTIDO e o resto e' efemero.
//! Misturar os dois e' exatamente como uma senha acaba em disco.
//!
//! # Estrutura
//!
//! - [`store`]: `.kinein/datasources.json`, com `schemaVersion`.
//! - [`secret`]: o tipo que nao se imprime, e a politica de onde buscar.
//! - [`connection`]: o unico lugar que fala com um servidor de banco.
//! - [`discover`]: o que responde nesta maquina (`0.124.0`).
//! - [`create`]: um banco onde nao havia (`0.124.0`).
//! - [`introspect`]: o que existe DENTRO do banco.
//! - [`mongo`]: o unico lugar que fala com um `MongoDB`.
//! - [`mongo_infer`]: dobrar documentos num mapa de campos, sem rede.
//!
//! # Nao confundir com `crate::db`
//!
//! `crate::db` e' a persistencia LOCAL da IDE (rascunhos em `SQLite`, a rede
//! de seguranca do `DocsPublic/seguranca/23`). Este modulo e' o banco DO AUTOR. Os
//! dois dizem "banco" e nao tem nada a ver um com o outro.

pub mod connection;
pub mod create;
pub mod discover;
pub mod introspect;
pub mod mongo;
pub mod mongo_infer;
pub mod query;
pub mod secret;
pub mod sqlite;
mod store;

use std::path::Path;

use kinein_protocol::{DataSourceEngine, DataSourceProfile};

pub use secret::{Secret, SecretPlan};

/// Porta padrao do `PostgreSQL`, usada quando o perfil nao informa outra.
pub const DEFAULT_PORT: u16 = 5432;

/// Le o catalogo do workspace, ja ordenado por nome.
#[must_use]
pub fn list(root: &Path) -> Vec<DataSourceProfile> {
    let mut profiles = store::load(root);
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    profiles
}

/// Valida um perfil vindo da UI.
///
/// A mensagem e' o que o autor vai ler; por isso ela diz o que fazer, nao o
/// que esta' errado em abstrato.
///
/// # Errors
/// Devolve a primeira falha encontrada, na ordem em que o formulario e' lido.
pub fn validate(profile: &DataSourceProfile) -> Result<(), String> {
    if profile.name.trim().is_empty() {
        return Err("de um nome ao perfil — e' por ele que a IDE o mostra".to_owned());
    }
    // CADA MOTOR COBRA O QUE ELE PEDE. Um arquivo `.db` nao tem host, porta
    // nem usuario; exigir os tres seria pedir ao autor que preenchesse o que
    // nao existe — e deixar campos vazios na tela e' a forma como a maioria
    // das IDEs trata SQLite (decisao do autor, 2026-09-04).
    if profile.engine == DataSourceEngine::Sqlite {
        return if profile.database.trim().is_empty() {
            Err("informe o caminho do arquivo .db".to_owned())
        } else {
            Ok(())
        };
    }
    if profile.host.trim().is_empty() {
        return Err("informe o host (use `localhost` para um banco nesta maquina)".to_owned());
    }
    // O MONGO NAO EXIGE USUARIO. Um servidor local sem autenticacao habilitada
    // e' o caso comum de desenvolvimento, e exigir o campo seria a IDE pedindo
    // o que o motor nao pede — o mesmo erro que a tela do SQLite corrigiu.
    if profile.engine == DataSourceEngine::Mongo {
        return if profile.database.trim().is_empty() {
            Err("informe o nome do banco".to_owned())
        } else {
            Ok(())
        };
    }
    if profile.port == 0 {
        return Err(format!(
            "porta invalida — o padrao do `PostgreSQL` e' {DEFAULT_PORT}"
        ));
    }
    if profile.database.trim().is_empty() {
        return Err("informe o nome do banco".to_owned());
    }
    if profile.user.trim().is_empty() {
        return Err("informe o usuario que conecta".to_owned());
    }
    Ok(())
}

/// Grava um perfil, criando ou SUBSTITUINDO o de mesmo nome.
///
/// O nome e' a identidade: salvar duas vezes com o mesmo nome edita, nao
/// duplica. Sem isso, corrigir uma porta errada deixaria dois perfis quase
/// iguais no painel e nenhum jeito de saber qual conecta.
///
/// # Errors
/// Perfil invalido ([`validate`]) ou falha de escrita em disco.
pub fn save(root: &Path, profile: &DataSourceProfile) -> Result<Vec<DataSourceProfile>, String> {
    validate(profile)?;
    let mut profiles = store::load(root);
    let normalizado = normalize(profile);
    match profiles
        .iter()
        .position(|existente| existente.name == normalizado.name)
    {
        Some(indice) => profiles[indice] = normalizado,
        None => profiles.push(normalizado),
    }
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    store::save(root, &profiles)?;
    Ok(profiles)
}

/// Remove o perfil de nome `name`. Remover o que nao existe NAO e' erro.
///
/// Devolver o catalogo em vez de "removi/nao removi" mantem a UI correta sem
/// ela precisar adivinhar: dois cliques rapidos no mesmo botao terminam no
/// mesmo estado.
///
/// # Errors
/// Falha de escrita em disco.
pub fn remove(root: &Path, name: &str) -> Result<Vec<DataSourceProfile>, String> {
    let mut profiles = store::load(root);
    profiles.retain(|profile| profile.name != name);
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    store::save(root, &profiles)?;
    Ok(profiles)
}

/// Tira espaco das bordas dos campos de texto.
///
/// Espaco no fim de um host colado de um chat e' erro comum, e o erro que ele
/// produz ("host nao encontrado") nao mostra o espaco.
fn normalize(profile: &DataSourceProfile) -> DataSourceProfile {
    DataSourceProfile {
        engine: profile.engine,
        name: profile.name.trim().to_owned(),
        host: profile.host.trim().to_owned(),
        port: profile.port,
        database: profile.database.trim().to_owned(),
        user: profile.user.trim().to_owned(),
        secret_source: profile.secret_source,
        secret_variable: profile
            .secret_variable
            .as_deref()
            .map(str::trim)
            .filter(|valor| !valor.is_empty())
            .map(str::to_owned),
        // A amostra so' vale para motor sem esquema fixo, e o valor e' preso
        // entre 1 e o teto: um perfil pedindo 50.000 documentos amostraria a
        // colecao inteira, que e' o oposto do que amostrar significa.
        sample_size: profile
            .sample_size
            .map(|valor| valor.clamp(1, mongo::MAX_SAMPLE)),
        // TLS so' vale para o PostgreSQL; `disable` e' o mesmo que ausente.
        tls: profile.tls.filter(|t| {
            *t != kinein_protocol::DataSourceTls::Disable
                && profile.engine == DataSourceEngine::Postgres
        }),
        ca_file: profile
            .ca_file
            .as_deref()
            .map(str::trim)
            .filter(|valor| !valor.is_empty())
            .map(str::to_owned),
    }
}

#[cfg(test)]
mod tests {
    use kinein_protocol::DataSourceEngine;
    use kinein_protocol::SecretSource;

    use super::*;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-datasource")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn perfil(name: &str) -> DataSourceProfile {
        DataSourceProfile {
            engine: DataSourceEngine::Postgres,
            name: name.to_owned(),
            host: "localhost".to_owned(),
            port: DEFAULT_PORT,
            database: "app".to_owned(),
            user: "postgres".to_owned(),
            secret_source: SecretSource::Automatic,
            secret_variable: None,
            sample_size: None,
            tls: None,
            ca_file: None,
        }
    }

    #[test]
    fn campo_vazio_reprova_com_mensagem_acionavel() {
        let mut sem_nome = perfil("");
        sem_nome.name = "   ".to_owned();
        assert!(validate(&sem_nome).unwrap_err().contains("nome"));

        let mut sem_host = perfil("local");
        sem_host.host = String::new();
        assert!(validate(&sem_host).unwrap_err().contains("localhost"));

        let mut porta_zero = perfil("local");
        porta_zero.port = 0;
        assert!(validate(&porta_zero).unwrap_err().contains("5432"));

        let mut sem_banco = perfil("local");
        sem_banco.database = String::new();
        assert!(validate(&sem_banco).is_err());

        let mut sem_usuario = perfil("local");
        sem_usuario.user = String::new();
        assert!(validate(&sem_usuario).is_err());

        assert!(validate(&perfil("local")).is_ok());
    }

    #[test]
    fn salvar_com_o_mesmo_nome_edita_em_vez_de_duplicar() {
        let root = temp_root("edita");
        save(&root, &perfil("local")).unwrap();

        let mut corrigido = perfil("local");
        corrigido.port = 6543;
        let catalogo = save(&root, &corrigido).unwrap();

        assert_eq!(catalogo.len(), 1, "corrigir a porta duplicou o perfil");
        assert_eq!(catalogo[0].port, 6543);
    }

    #[test]
    fn catalogo_volta_ordenado_por_nome() {
        let root = temp_root("ordem");
        save(&root, &perfil("staging")).unwrap();
        save(&root, &perfil("local")).unwrap();
        let nomes: Vec<_> = list(&root).into_iter().map(|p| p.name).collect();
        assert_eq!(nomes, vec!["local".to_owned(), "staging".to_owned()]);
    }

    #[test]
    fn espaco_nas_bordas_some_ao_salvar() {
        let root = temp_root("trim");
        let mut colado = perfil("  local  ");
        colado.host = " db.example.com ".to_owned();
        colado.secret_variable = Some("   ".to_owned());
        let catalogo = save(&root, &colado).unwrap();

        assert_eq!(catalogo[0].name, "local");
        assert_eq!(catalogo[0].host, "db.example.com");
        assert_eq!(
            catalogo[0].secret_variable, None,
            "variavel so' com espaco tinha que virar ausente"
        );
    }

    #[test]
    fn remover_o_que_nao_existe_nao_e_erro() {
        let root = temp_root("remove");
        save(&root, &perfil("local")).unwrap();

        assert_eq!(remove(&root, "local").unwrap().len(), 0);
        assert_eq!(
            remove(&root, "local").unwrap().len(),
            0,
            "remover duas vezes tinha que terminar no mesmo estado"
        );
    }

    /// O `crate::db` guarda rascunho da IDE; este dominio guarda o banco DO
    /// AUTOR. Sao arquivos diferentes, e trocar um pelo outro apagaria
    /// rascunho nao salvo.
    #[test]
    fn o_catalogo_nao_encosta_no_banco_local_da_ide() {
        let root = temp_root("vizinhos");
        save(&root, &perfil("local")).unwrap();
        assert!(root.join(".kinein/datasources.json").exists());
        assert!(!root.join(".kinein/kinein.db").exists());
    }
}
