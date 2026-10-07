//! Leases contam trabalhadores por destino; a barreira espera fora do despacho.
//! Nao guarda driver, senha, SQL ou perfil e nao interrompe escrita ja' aceita.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};

const UNAVAILABLE: &str = "O registro de conexões está indisponível.";
const CLOSING: &str = "A conexão está desconectando; aguarde o encerramento.";
type Key = (PathBuf, String);

#[derive(Debug, Default)]
struct Entry {
    active: usize,
    closing: bool,
}

#[derive(Debug, Default)]
struct Registry {
    entries: Mutex<HashMap<Key, Entry>>,
    changed: Condvar,
}

/// Cada core tem seu registro; leases continuam vivas ate' o trabalhador sair.
#[derive(Debug, Clone, Default)]
pub struct Session {
    registry: Arc<Registry>,
}

#[derive(Debug)]
/// Reserva uma operacao antes de seu trabalhador comecar.
pub struct Operation {
    session: Session,
    key: Key,
}

#[derive(Debug)]
/// Barreira por destino; nenhum outro pedido inicia ate' sua liberacao.
pub struct Disconnect {
    session: Session,
    key: Key,
}

impl Session {
    /// Reserva uma operacao sem conectar.
    ///
    /// # Errors
    /// Registro indisponivel ou destino em encerramento.
    pub fn begin(&self, root: &Path, name: &str) -> Result<Operation, &'static str> {
        let key = (root.to_path_buf(), name.to_owned());
        let mut entries = self.registry.entries.lock().map_err(|_| UNAVAILABLE)?;
        let entry = entries.entry(key.clone()).or_default();
        if entry.closing {
            return Err(CLOSING);
        }
        entry.active += 1;
        drop(entries);
        Ok(Operation {
            session: self.clone(),
            key,
        })
    }

    /// Recusa novas operacoes enquanto as leases anteriores terminam.
    ///
    /// # Errors
    /// Registro indisponivel ou outra desconexao no mesmo destino.
    pub fn disconnect(&self, root: &Path, name: &str) -> Result<Disconnect, &'static str> {
        let key = (root.to_path_buf(), name.to_owned());
        let mut entries = self.registry.entries.lock().map_err(|_| UNAVAILABLE)?;
        let entry = entries.entry(key.clone()).or_default();
        if entry.closing {
            return Err(CLOSING);
        }
        entry.closing = true;
        drop(entries);
        Ok(Disconnect {
            session: self.clone(),
            key,
        })
    }
}

impl Disconnect {
    /// Mutex liberado na espera; nenhum outro destino fica bloqueado por rede.
    ///
    /// # Errors
    /// O mutex do registro foi envenenado.
    pub fn wait(&self) -> Result<(), &'static str> {
        let entries = self
            .session
            .registry
            .entries
            .lock()
            .map_err(|_| UNAVAILABLE)?;
        let entries = self
            .session
            .registry
            .changed
            .wait_while(entries, |entries| {
                entries.get(&self.key).is_some_and(|entry| entry.active > 0)
            })
            .map_err(|_| UNAVAILABLE)?;
        drop(entries);
        Ok(())
    }
}

impl Drop for Operation {
    fn drop(&mut self) {
        if let Ok(mut entries) = self.session.registry.entries.lock() {
            if let Some(entry) = entries.get_mut(&self.key) {
                entry.active = entry.active.saturating_sub(1);
                if entry.active == 0 && !entry.closing {
                    entries.remove(&self.key);
                }
            }
            self.session.registry.changed.notify_all();
        }
    }
}

impl Drop for Disconnect {
    fn drop(&mut self) {
        if let Ok(mut entries) = self.session.registry.entries.lock()
            && let Some(entry) = entries.get_mut(&self.key)
        {
            entry.closing = false;
            if entry.active == 0 {
                entries.remove(&self.key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn disconnect_waits_for_every_lease_and_isolates_destinations() {
        let session = Session::default();
        let root = Path::new("/project");
        let first = session.begin(root, "__proto__").unwrap();
        let second = session.begin(root, "__proto__").unwrap();
        let closing = session.disconnect(root, "__proto__").unwrap();
        assert!(session.begin(root, "__proto__").is_err());
        assert!(session.disconnect(root, "__proto__").is_err());
        assert!(session.begin(root, "other").is_ok());
        assert!(session.begin(Path::new("/other"), "__proto__").is_ok());
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            closing.wait().unwrap();
            sender.send(closing).unwrap();
        });
        assert!(receiver.recv_timeout(Duration::from_millis(30)).is_err());
        drop(first);
        assert!(receiver.recv_timeout(Duration::from_millis(30)).is_err());
        drop(second);
        let closing = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(session.begin(root, "__proto__").is_err());
        drop(closing);
        assert!(session.begin(root, "__proto__").is_ok());
        worker.join().unwrap();
        assert!(session.registry.entries.lock().unwrap().is_empty());
    }
}
