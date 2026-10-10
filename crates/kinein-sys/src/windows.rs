//! A API do Windows atras de uma interface segura. Cada bloco `unsafe` diz, no
//! `SAFETY`, por que a chamada e' valida.
#![allow(unsafe_code)]

use std::{
    ffi::c_void,
    io,
    mem::{size_of, zeroed},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Component, Path, Prefix},
    process::Child,
    ptr,
};

use windows_sys::Win32::{
    Foundation::INVALID_HANDLE_VALUE,
    Storage::FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
        },
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject, TerminateJobObject,
        },
        Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME},
    },
};

/// Acima disto a API so' aceita o caminho na forma verbatim (`\\?\`). O teto do
/// `MAX_PATH` e' 260; 248 e' o de pasta, e vale para os dois casos.
const SHORT_PATH_LIMIT: usize = 248;

/// O tamanho de `T` para os campos `cb...`/`dwSize` da API.
fn size_u32<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap_or(u32::MAX)
}

/// Um Job Object: o grupo de processos que morre junto.
///
/// Com `KILL_ON_JOB_CLOSE`, fechar o ultimo handle mata todo processo do job —
/// inclusive quando quem o segura morre num crash. E' o par do grupo de
/// processo do Unix, mais forte que ele: um descendente nao sai do job, nem
/// criando sessao propria.
#[derive(Debug)]
pub struct Job {
    handle: OwnedHandle,
}

impl Job {
    /// Um job novo, sem nome, com `KILL_ON_JOB_CLOSE`.
    ///
    /// # Errors
    /// O sistema recusou criar ou configurar o job.
    pub fn new() -> io::Result<Self> {
        // SAFETY: atributos nulos (o handle nao e' herdavel) e nome nulo (job
        // anonimo); a funcao so' le os dois ponteiros.
        let raw = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `raw` acabou de vir valido do sistema, e ninguem mais o possui.
        let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
        // SAFETY: a estrutura e' so' de inteiros e ponteiros; zero e' um valor
        // valido para todos os campos, e e' o que a documentacao manda.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: o handle e' um job valido; o ponteiro aponta para a estrutura
        // do tipo e do tamanho que a classe pede, viva durante a chamada.
        let ok = unsafe {
            SetInformationJobObject(
                handle.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                ptr::from_ref(&limits).cast::<c_void>(),
                size_u32::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { handle })
    }

    /// Poe `child` no job. Os processos que ele criar depois entram junto.
    ///
    /// # Errors
    /// O processo ja' saiu, ou o sistema recusou a associacao.
    pub fn assign(&self, child: &Child) -> io::Result<()> {
        // SAFETY: os dois handles sao validos durante a chamada: o do job e'
        // nosso, e o do processo e' do `Child`, que o segura ate' o drop.
        let ok =
            unsafe { AssignProcessToJobObject(self.handle.as_raw_handle(), child.as_raw_handle()) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Mata todo processo do job, sem esperar.
    ///
    /// # Errors
    /// O sistema recusou o pedido.
    pub fn terminate(&self) -> io::Result<()> {
        // SAFETY: o handle e' um job valido e nosso.
        let ok = unsafe { TerminateJobObject(self.handle.as_raw_handle(), 1) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

/// Retoma as threads do processo `pid`, criado com `CREATE_SUSPENDED`.
///
/// # Errors
/// O sistema recusou a foto das threads, ou nenhuma thread do processo foi
/// retomada.
pub fn resume_process(pid: u32) -> io::Result<()> {
    // SAFETY: `TH32CS_SNAPTHREAD` com pid 0 e' o uso documentado (todas as
    // threads do sistema; o filtro por processo e' feito abaixo).
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `raw` acabou de vir valido do sistema, e ninguem mais o possui.
    let snapshot = unsafe { OwnedHandle::from_raw_handle(raw) };
    // SAFETY: a estrutura e' so' de inteiros; zero e' valido em todos os
    // campos, e o `dwSize` e' preenchido logo abaixo, como a API pede.
    let mut entry: THREADENTRY32 = unsafe { zeroed() };
    entry.dwSize = size_u32::<THREADENTRY32>();
    let mut resumed = 0_usize;
    // SAFETY: a foto e' valida e a estrutura tem o `dwSize` certo.
    let mut more = unsafe { Thread32First(snapshot.as_raw_handle(), &raw mut entry) } != 0;
    while more {
        if entry.th32OwnerProcessID == pid {
            // SAFETY: o id vem da foto; o acesso pedido e' o minimo para retomar.
            let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
            if !thread.is_null() {
                // SAFETY: `thread` acabou de vir valido, e ninguem mais o possui.
                let thread = unsafe { OwnedHandle::from_raw_handle(thread) };
                // SAFETY: handle de thread aberto com `THREAD_SUSPEND_RESUME`.
                if unsafe { ResumeThread(thread.as_raw_handle()) } != u32::MAX {
                    resumed += 1;
                }
            }
        }
        // SAFETY: como no `Thread32First`.
        more = unsafe { Thread32Next(snapshot.as_raw_handle(), &raw mut entry) } != 0;
    }
    if resumed == 0 {
        return Err(io::Error::other(format!(
            "nenhuma thread do processo {pid} foi retomada"
        )));
    }
    Ok(())
}

/// Move `from` para `to` sem nunca sobrescrever, atomico no mesmo volume.
///
/// Com `to` existente, falha com [`io::ErrorKind::AlreadyExists`]; entre
/// volumes, falha (como o `EXDEV` do Unix). Vale para arquivo e para pasta.
///
/// # Errors
/// O destino existe, a origem nao existe, ou o sistema recusou.
pub fn rename_noreplace(from: &Path, to: &Path) -> io::Result<()> {
    let from = wide(from);
    let to = wide(to);
    // SAFETY: os dois buffers terminam em zero e vivem durante a chamada. Sem
    // `MOVEFILE_REPLACE_EXISTING`, o sistema recusa destino existente.
    let ok = unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), MOVEFILE_WRITE_THROUGH) };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// O caminho em UTF-16 terminado em zero. Longo demais para a forma comum, vai
/// na forma verbatim (`\\?\`), que nao normaliza: as barras viram `\`.
fn wide(path: &Path) -> Vec<u16> {
    let text: Vec<u16> = path.as_os_str().encode_wide().collect();
    let verbatim_prefix = match path.components().next() {
        _ if text.len() < SHORT_PATH_LIMIT => None,
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(_) => Some((r"\\?\", 0)),
            Prefix::UNC(..) => Some((r"\\?\UNC", 1)),
            _ => None,
        },
        _ => None,
    };
    let mut out: Vec<u16> = match verbatim_prefix {
        Some((prefix, skip)) => prefix
            .encode_utf16()
            .chain(text.iter().skip(skip).copied())
            .map(|unit| {
                if unit == u16::from(b'/') {
                    u16::from(b'\\')
                } else {
                    unit
                }
            })
            .collect(),
        None => text,
    };
    out.push(0);
    out
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

    use super::{Job, rename_noreplace};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kinein-sys-{}-{name}", std::process::id()));
        drop(fs::remove_dir_all(&dir));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn rename_noreplace_moves_a_file_and_a_folder_and_refuses_an_existing_target() {
        let dir = temp_dir("rename");
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(dir.join("b.txt"), "b").unwrap();
        let error = rename_noreplace(&dir.join("a.txt"), &dir.join("b.txt")).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read_to_string(dir.join("b.txt")).unwrap(),
            "b",
            "destino intacto"
        );
        rename_noreplace(&dir.join("a.txt"), &dir.join("c.txt")).unwrap();
        assert_eq!(fs::read_to_string(dir.join("c.txt")).unwrap(), "a");
        fs::create_dir(dir.join("pasta")).unwrap();
        fs::create_dir(dir.join("outra")).unwrap();
        let error = rename_noreplace(&dir.join("pasta"), &dir.join("outra")).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        rename_noreplace(&dir.join("pasta"), &dir.join("nova")).unwrap();
        assert!(dir.join("nova").is_dir());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_long_path_is_moved_through_the_verbatim_form() {
        let dir = temp_dir("longo");
        let mut deep = dir.clone();
        while deep.as_os_str().len() < 300 {
            deep.push("uma-pasta-com-nome-comprido");
        }
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("a.txt"), "a").unwrap();
        rename_noreplace(&deep.join("a.txt"), &deep.join("b.txt")).unwrap();
        assert!(deep.join("b.txt").is_file());
        fs::remove_dir_all(dir).unwrap();
    }

    /// O filho cria um NETO e sai: fechar o job tem de matar o neto tambem,
    /// que e' o que um grupo de processo do Unix nao garante.
    #[test]
    fn closing_the_job_kills_a_grandchild_that_outlived_its_parent() {
        let marker = temp_dir("job").join("neto.pid");
        let script = format!(
            "$p = Start-Process -FilePath ping -ArgumentList '-n','60','127.0.0.1' -WindowStyle Hidden -PassThru; Set-Content -Path '{}' -Value $p.Id",
            marker.display()
        );
        let job = Job::new().unwrap();
        let mut child = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        job.assign(&child).unwrap();
        child.wait().unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        let grandchild = loop {
            if let Ok(text) = fs::read_to_string(&marker)
                && let Ok(pid) = text.trim().parse::<u32>()
            {
                break pid;
            }
            assert!(Instant::now() < deadline, "o neto nao nasceu");
            std::thread::sleep(Duration::from_millis(100));
        };
        assert!(
            alive(grandchild),
            "o neto devia estar vivo antes de fechar o job"
        );
        drop(job);
        let deadline = Instant::now() + Duration::from_secs(10);
        while alive(grandchild) {
            assert!(Instant::now() < deadline, "o neto sobreviveu ao job");
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn alive(pid: u32) -> bool {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .is_ok_and(|out| String::from_utf8_lossy(&out.stdout).contains(&pid.to_string()))
    }
}
