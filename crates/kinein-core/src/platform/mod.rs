//! O que difere por sistema operacional, num lugar so' (DocsPublic/roadmaps/60
//! §3.2, fatia W1 do porte para o Windows).
//!
//! Os dominios chamam estas funcoes em vez de usar `std::os::unix`, `rustix`
//! ou `cfg(windows)` por conta propria. No Unix cada uma e' o comportamento de
//! antes do porte (2026-10-09), movido sem mudar. No Windows e' o equivalente
//! sem `unsafe` (o crate e' `forbid(unsafe_code)`), com o limite dito na
//! funcao.
//!
//! Os dominios que ainda nao rodam no Windows (serial, consoles do Banco) nao
//! passam por aqui: eles tem um stub `#[cfg(windows)]` que responde "nao
//! suportado no Windows" ate' a fatia deles (60 §1).

// A API e' do crate e nada dela e' publico: `pub(crate)` e' a visibilidade
// certa, e `pub` seria reprovado pelo `unreachable_pub` do workspace. O lint do
// clippy discorda dos dois; o mesmo conflito esta' resolvido assim no
// `lang/indent.rs`.
#![allow(clippy::redundant_pub_crate)]

use std::{
    ffi::{OsStr, OsString},
    fs::{DirBuilder, File, OpenOptions, Permissions},
    io,
    path::Path,
    process::{Child, Command},
};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix as imp;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as imp;

/// Por que um no' nao pode ser aberto sem seguir link.
#[derive(Debug)]
pub(crate) enum NoFollow {
    /// O no' e' link. No Unix, link simbolico em qualquer componente; no
    /// Windows, qualquer ponto de reparse (link simbolico, juncao).
    Link,
    /// O caminho tem `..` ou nao e' absoluto.
    InvalidPath,
    /// Outra falha de E/S.
    Io(io::Error),
}

/// Abre o caminho absoluto `path` sem seguir link em nenhum componente.
///
/// No Unix, cada componente e' aberto com `openat` e `O_NOFOLLOW` a partir de
/// `/`. No Windows, cada prefixo do caminho e' aberto sem seguir ponto de
/// reparse e conferido no proprio handle. Limite no Windows: entre a conferencia
/// de uma pasta e a abertura do filho, a pasta pode ser trocada por uma juncao;
/// o filho em si e' sempre conferido no handle que se le.
///
/// # Errors
/// [`NoFollow::Link`] se algum componente e' link; [`NoFollow::InvalidPath`]
/// para `..`; [`NoFollow::Io`] para o resto.
pub(crate) fn open_nofollow(path: &Path) -> Result<File, NoFollow> {
    imp::open_nofollow(path)
}

/// Abre o filho `name` da pasta `parent`, aberta por esta API em `parent_path`,
/// sem seguir link.
///
/// # Errors
/// Os mesmos de [`open_nofollow`].
pub(crate) fn open_child_nofollow(
    parent: &File,
    parent_path: &Path,
    name: &OsStr,
) -> Result<File, NoFollow> {
    imp::open_child_nofollow(parent, parent_path, name)
}

/// Os nomes dentro da pasta `dir`, aberta por esta API em `dir_path`, sem `.` e
/// `..`. So' os nomes: o chamador abre um filho de cada vez.
///
/// # Errors
/// Falha ao ler a pasta.
pub(crate) fn child_names(dir: &File, dir_path: &Path) -> io::Result<Vec<OsString>> {
    imp::child_names(dir, dir_path)
}

/// Move `from` para `to` sem nunca sobrescrever: com `to` existente, falha com
/// [`io::ErrorKind::AlreadyExists`].
///
/// No Unix e' um `renameat2` com `RENAME_NOREPLACE`, atomico. No Windows, um
/// arquivo e' publicado por `hard_link` (que falha se `to` existe) seguido da
/// remocao de `from`; uma pasta, por `rename` depois de conferir que `to` nao
/// existe. Limite no Windows: uma pasta criada em `to` entre a conferencia e o
/// `rename` faz o `rename` falhar, mas um ARQUIVO criado ali seria substituido.
///
/// # Errors
/// [`io::ErrorKind::AlreadyExists`] se `to` existe; o erro do sistema no resto.
pub(crate) fn rename_noreplace(from: &Path, to: &Path) -> io::Result<()> {
    imp::rename_noreplace(from, to)
}

/// Faz `options` criar um arquivo que so' o usuario le: `0600` no Unix. No
/// Windows nada muda: o arquivo herda a ACL da pasta, e a pasta do usuario
/// (`%APPDATA%`, `%LOCALAPPDATA%`, o `%TEMP%`) so' da' acesso ao usuario, ao
/// SYSTEM e aos Administradores (decisao D3 do autor, 60 §3.1).
pub(crate) fn owner_only_file(options: &mut OpenOptions) {
    imp::owner_only_file(options);
}

/// Um construtor de pasta que so' o usuario abre: `0700` no Unix; no Windows,
/// a ACL herdada, como em [`owner_only_file`].
#[must_use]
pub(crate) fn owner_only_dir_builder() -> DirBuilder {
    imp::owner_only_dir_builder()
}

/// Fecha para o usuario uma pasta que ja' existia mais aberta: `0700` no Unix.
/// No Windows so' confere que ela existe; a ACL e' a herdada (D3).
///
/// # Errors
/// A pasta nao existe ou a permissao nao pode ser mudada.
pub(crate) fn restrict_dir_to_owner(path: &Path) -> io::Result<()> {
    imp::restrict_dir_to_owner(path)
}

/// `permissions` com escrita para o dono, para poder apagar uma copia
/// interrompida: `u+rwx` no Unix; sem o atributo somente-leitura no Windows.
#[must_use]
pub(crate) fn owner_writable(permissions: Permissions) -> Permissions {
    imp::owner_writable(permissions)
}

/// As variaveis que um processo de longa vida herda quando o ambiente e'
/// explicito (`owned_child::Env::Allowlist`); o resto vem do chamador. No
/// Windows entram as que o proprio sistema precisa para um programa subir
/// (`SystemRoot`, `windir`, `ComSpec`, `PATHEXT`) e as pastas do usuario.
pub(crate) const INHERITED_ENV: &[&str] = imp::INHERITED_ENV;

/// O grupo de um processo filho: encerra-lo alcanca os descendentes.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Group(imp::GroupId);

/// Faz `command` criar o proprio grupo. No Unix e' o `process_group(0)`. No
/// Windows o grupo e' a arvore de processos (ver [`kill_group`]), e o filho nao
/// abre janela de console.
pub(crate) fn own_group(command: &mut Command) {
    imp::own_group(command);
}

/// O grupo de `child`, criado por um `command` que passou por [`own_group`].
#[must_use]
pub(crate) fn group_of(child: &Child) -> Option<Group> {
    imp::group_of(child).map(Group)
}

/// Pede ao grupo que termine (`SIGTERM` no Unix) e diz se esse pedido existe
/// neste sistema. No Windows nao existe para um processo sem janela, e a
/// resposta e' `false`: o chamador passa direto para [`kill_group`].
pub(crate) fn terminate_group(group: Group) -> bool {
    imp::terminate_group(group.0)
}

/// Mata o grupo, sem colher. No Unix e' `SIGKILL` no grupo. No Windows e'
/// `taskkill /T /F`, que mata o processo e a arvore dele. Limite no Windows: um
/// descendente cujo pai ja' saiu nao esta' mais na arvore e escapa.
pub(crate) fn kill_group(group: Group) {
    imp::kill_group(group.0);
}
