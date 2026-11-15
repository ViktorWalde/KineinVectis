//! O lançador: todo processo nasce aqui, com PDEATHSIG e, para o `nvim`, a
//! camada 1 do Landlock.
//!
//! A camada 1 trata o direito de executar e libera só dois arquivos: o binário
//! do `nvim` (o próprio `execve` dele) e o interpretador ELF que o kernel abre
//! junto. A camada 2, dentro do `nvim` (o `contain()` do runtime Lua), fecha
//! os dois. O PDEATHSIG faz o filho morrer com o supervisor, mesmo num
//! `kill -9`, que não deixa o supervisor limpar nada.

use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const SYS_LANDLOCK_CREATE_RULESET: libc::c_long = 444;
const SYS_LANDLOCK_ADD_RULE: libc::c_long = 445;
const SYS_LANDLOCK_RESTRICT_SELF: libc::c_long = 446;
const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1;
const LANDLOCK_RULE_PATH_BENEATH: libc::c_long = 1;
const LANDLOCK_CREATE_RULESET_VERSION: libc::c_long = 1;

#[repr(C)]
struct RulesetAttr {
    handled_access_fs: u64,
}

#[repr(C, packed)]
struct PathBeneathAttr {
    allowed_access: u64,
    parent_fd: i32,
}

/// A versão do Landlock do kernel, ou o erro que a impede.
pub fn landlock_abi() -> Result<i64, String> {
    // SAFETY: a chamada só consulta a versão; ponteiro nulo e tamanho 0 são o
    // contrato documentado do LANDLOCK_CREATE_RULESET_VERSION.
    let abi = unsafe {
        libc::syscall(
            SYS_LANDLOCK_CREATE_RULESET,
            std::ptr::null::<RulesetAttr>(),
            0usize,
            LANDLOCK_CREATE_RULESET_VERSION,
        )
    };
    if abi < 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    Ok(abi)
}

/// O interpretador que o kernel carrega com o binário (`PT_INTERP`).
pub fn elf_interpreter(binary: &Path) -> io::Result<Option<PathBuf>> {
    let mut data = Vec::new();
    File::open(binary)?.read_to_end(&mut data)?;
    let invalid = || io::Error::new(io::ErrorKind::InvalidData, "ELF inválido");
    if data.len() < 64 || &data[..4] != b"\x7fELF" || data[4] != 2 {
        return Err(invalid());
    }
    let u16_at = |o: usize| u16::from_le_bytes([data[o], data[o + 1]]) as usize;
    let u32_at = |o: usize| u32::from_le_bytes(data[o..o + 4].try_into().unwrap());
    let u64_at = |o: usize| u64::from_le_bytes(data[o..o + 8].try_into().unwrap()) as usize;
    let phoff = u64_at(0x20);
    let phentsize = u16_at(0x36);
    let phnum = u16_at(0x38);
    for i in 0..phnum {
        let ph = phoff + i * phentsize;
        if ph + 56 > data.len() {
            return Err(invalid());
        }
        if u32_at(ph) == 3 {
            let off = u64_at(ph + 8);
            let size = u64_at(ph + 32);
            let raw = data.get(off..off + size).ok_or_else(invalid)?;
            let text = raw.split(|b| *b == 0).next().unwrap_or_default();
            return Ok(Some(PathBuf::from(String::from_utf8_lossy(text).into_owned())));
        }
    }
    Ok(None)
}

/// O que o filho do `nvim` precisa para a camada 1: os arquivos que podem ser
/// executados, já abertos em O_PATH no pai (o filho entre o fork e o exec não
/// deve alocar nem resolver caminho).
pub struct ExecAllowList {
    files: Vec<OwnedFd>,
    pub paths: Vec<PathBuf>,
}

impl ExecAllowList {
    pub fn for_binary(binary: &Path) -> io::Result<Self> {
        let binary = binary.canonicalize()?;
        let mut paths = vec![binary.clone()];
        if let Some(interp) = elf_interpreter(&binary)? {
            paths.push(interp.canonicalize()?);
        }
        let mut files = Vec::new();
        for p in &paths {
            let c = CString::new(p.as_os_str().as_bytes())?;
            // SAFETY: caminho terminado em zero; o descritor vira OwnedFd logo
            // abaixo e fecha com ele.
            let fd = unsafe { libc::open(c.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: fd acabou de ser aberto e não tem outro dono.
            files.push(unsafe { <OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(fd) });
        }
        Ok(Self { files, paths })
    }
}

fn set_parent_death_signal(expected_parent: libc::pid_t) -> io::Result<()> {
    // SAFETY: prctl e getppid são seguros entre fork e exec.
    unsafe {
        if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
            return Err(io::Error::last_os_error());
        }
        // O pai pode ter morrido antes do prctl: aí o sinal nunca viria.
        if libc::getppid() != expected_parent {
            libc::_exit(1);
        }
    }
    Ok(())
}

/// Prepara um processo que morre com o supervisor.
pub fn supervised(command: &mut Command) {
    let parent = std::process::id() as libc::pid_t;
    // SAFETY: o fechamento só chama funções seguras entre fork e exec.
    unsafe {
        command.pre_exec(move || set_parent_death_signal(parent));
    }
}

/// Prepara o `nvim`: morre com o supervisor e só pode executar a lista.
pub fn supervised_without_exec(command: &mut Command, allow: &ExecAllowList) {
    let parent = std::process::id() as libc::pid_t;
    let fds: Vec<i32> = allow.files.iter().map(|f| f.as_raw_fd()).collect();
    // SAFETY: o fechamento só faz chamadas de sistema com memória já alocada no
    // pai (o vetor de descritores e as estruturas na pilha).
    unsafe {
        command.pre_exec(move || {
            set_parent_death_signal(parent)?;
            if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                return Err(io::Error::last_os_error());
            }
            let attr = RulesetAttr { handled_access_fs: LANDLOCK_ACCESS_FS_EXECUTE };
            let ruleset = libc::syscall(
                SYS_LANDLOCK_CREATE_RULESET,
                &attr as *const RulesetAttr,
                std::mem::size_of::<RulesetAttr>(),
                0u32,
            );
            if ruleset < 0 {
                return Err(io::Error::last_os_error());
            }
            for fd in &fds {
                let rule = PathBeneathAttr { allowed_access: LANDLOCK_ACCESS_FS_EXECUTE, parent_fd: *fd };
                if libc::syscall(
                    SYS_LANDLOCK_ADD_RULE,
                    ruleset,
                    LANDLOCK_RULE_PATH_BENEATH,
                    &rule as *const PathBeneathAttr,
                    0u32,
                ) != 0
                {
                    return Err(io::Error::last_os_error());
                }
            }
            if libc::syscall(SYS_LANDLOCK_RESTRICT_SELF, ruleset, 0u32) != 0 {
                return Err(io::Error::last_os_error());
            }
            libc::close(ruleset as i32);
            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interpreter_of_a_dynamic_binary_is_found() {
        let interp = elf_interpreter(Path::new("/bin/true")).unwrap();
        assert!(interp.unwrap().to_string_lossy().contains("ld-linux"));
    }

    #[test]
    fn a_child_under_layer_one_cannot_exec_anything_outside_the_list() {
        let nvim_like = Path::new("/bin/true").canonicalize().unwrap();
        let allow = ExecAllowList::for_binary(&nvim_like).unwrap();
        // O filho é um `sh` liberado? Não: só o /bin/true e o ld.so estão na
        // lista, então o próprio exec do `sh` falha.
        let mut sh = Command::new("/bin/sh");
        sh.arg("-c").arg("exit 0");
        supervised_without_exec(&mut sh, &allow);
        let err = sh.status().unwrap_err();
        assert_eq!(err.raw_os_error(), Some(libc::EACCES), "{err}");
        // O binário da lista executa.
        let mut ok = Command::new(&nvim_like);
        supervised_without_exec(&mut ok, &allow);
        assert!(ok.status().unwrap().success());
    }
}
