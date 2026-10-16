//! O nome do servidor resolvido com PRAZO, fora do Tokio (passo 15b,
//! `roadmaps/59` §5.6.1).
//!
//! O `tokio-postgres` resolve o `host` pelo `lookup_host` do Tokio: o
//! `getaddrinfo` do sistema numa thread bloqueante do runtime, sem prazo (o
//! `connect_timeout` cobre so' o TCP). Medido em 2026-10-15: com um resolvedor
//! que nao responde, o `getaddrinfo` trava 12 s (`timeout:3 attempts:2`), e o
//! `Drop` do runtime da previa espera essa thread.
//!
//! Aqui o nome e' resolvido ANTES do driver, numa thread propria, e o driver
//! recebe os enderecos pelo `hostaddr` (um par `host`/`hostaddr` por
//! endereco): o `host` continua sendo o nome que o TLS verifica, e o driver
//! tenta os enderecos em ordem, como antes. A thread que estoura o prazo
//! termina sozinha quando o `getaddrinfo` volta; para nao acumularem, ha' um
//! teto de resolucoes pendentes, e a que passa dele e' recusada na hora.

use std::io;
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Quanto o resolvedor do sistema pode levar.
pub const RESOLVE_DEADLINE: Duration = Duration::from_secs(5);

/// Quantas resolucoes podem ficar pendentes no processo.
pub const MAX_PENDING: usize = 4;

/// Quem pergunta o nome ao sistema (o teste injeta um que dorme).
pub type Lookup = fn(&str, u16) -> io::Result<Vec<IpAddr>>;

/// O resolvedor com prazo e teto de pendentes.
#[derive(Debug, Clone)]
pub struct Resolver {
    lookup: Lookup,
    deadline: Duration,
    max_pending: usize,
    pending: Arc<AtomicUsize>,
}

/// O resolvedor do processo: o `getaddrinfo` do sistema, com o prazo e o teto.
#[must_use]
pub fn system() -> &'static Resolver {
    static SYSTEM: std::sync::OnceLock<Resolver> = std::sync::OnceLock::new();
    SYSTEM.get_or_init(|| Resolver::new(system_lookup, RESOLVE_DEADLINE, MAX_PENDING))
}

fn system_lookup(host: &str, port: u16) -> io::Result<Vec<IpAddr>> {
    Ok((host, port)
        .to_socket_addrs()?
        .map(|addr| addr.ip())
        .collect())
}

/// O que o host do perfil precisa: nada (socket Unix, IP literal, vazio), ou
/// o resolvedor.
fn literal(host: &str) -> Option<Vec<IpAddr>> {
    let host = host.trim();
    if host.is_empty() || host.starts_with('/') {
        return Some(Vec::new());
    }
    host.parse::<IpAddr>().ok().map(|ip| vec![ip])
}

/// Devolve a vaga quando a thread de resolucao termina.
struct Slot(Arc<AtomicUsize>);

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

impl Resolver {
    /// Um resolvedor com o seu proprio contador de pendentes.
    #[must_use]
    pub fn new(lookup: Lookup, deadline: Duration, max_pending: usize) -> Self {
        Self {
            lookup,
            deadline,
            max_pending,
            pending: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Quantas resolucoes estao pendentes agora.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.pending.load(Ordering::SeqCst)
    }

    /// Os enderecos de `host`, esperando no maximo o prazo. Vazio quando o
    /// driver nao precisa de endereco (socket Unix, host vazio).
    ///
    /// # Errors
    /// A frase para a tela: prazo estourado, teto de pendentes ou o nome que
    /// nao resolveu.
    pub fn addresses(&self, host: &str, port: u16) -> Result<Vec<IpAddr>, String> {
        if let Some(ready) = literal(host) {
            return Ok(ready);
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        self.spawn(host, port, move |answer| drop(sender.send(answer)))?;
        receiver
            .recv_timeout(self.deadline)
            .map_err(|_| self.expired(host))?
    }

    /// Como [`Resolver::addresses`], aguardado sem prender o runtime: o
    /// cancelamento de quem espera continua respondendo, e o `Drop` do
    /// runtime nao espera a thread.
    ///
    /// # Errors
    /// As mesmas de [`Resolver::addresses`].
    pub async fn addresses_async(&self, host: &str, port: u16) -> Result<Vec<IpAddr>, String> {
        if let Some(ready) = literal(host) {
            return Ok(ready);
        }
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.spawn(host, port, move |answer| drop(sender.send(answer)))?;
        match tokio::time::timeout(self.deadline, receiver).await {
            Ok(Ok(answer)) => answer,
            _ => Err(self.expired(host)),
        }
    }

    fn expired(&self, host: &str) -> String {
        format!(
            "o resolvedor de nomes do sistema nao respondeu em {} s para {}",
            self.deadline.as_secs(),
            host.trim()
        )
    }

    /// Ocupa uma vaga e pergunta numa thread propria; sem vaga, recusa.
    fn spawn(
        &self,
        host: &str,
        port: u16,
        deliver: impl FnOnce(Result<Vec<IpAddr>, String>) + Send + 'static,
    ) -> Result<(), String> {
        let taken = self.pending.fetch_add(1, Ordering::SeqCst);
        let slot = Slot(Arc::clone(&self.pending));
        if taken >= self.max_pending {
            return Err(format!(
                "o resolvedor de nomes do sistema nao esta' respondendo: {} resolucoes ainda \
                 pendentes; tente de novo em instantes",
                self.max_pending
            ));
        }
        let host = host.trim().to_owned();
        let shown = host.clone();
        let lookup = self.lookup;
        std::thread::Builder::new()
            .name("kinein-resolve".to_owned())
            .spawn(move || {
                let _slot = slot;
                deliver(match lookup(&host, port) {
                    Ok(found) if found.is_empty() => {
                        Err(format!("{host}: o nome nao resolveu para nenhum endereco"))
                    }
                    Ok(found) => Ok(dedup(found)),
                    Err(error) => Err(format!("{host}: nome nao resolvido ({error})")),
                });
            })
            .map(drop)
            .map_err(|error| format!("nao foi possivel resolver {shown}: {error}"))
    }
}

/// Os enderecos na ordem do resolvedor, sem repetir (o `getaddrinfo` devolve
/// o mesmo IP uma vez por tipo de socket).
fn dedup(found: Vec<IpAddr>) -> Vec<IpAddr> {
    let mut seen = Vec::with_capacity(found.len());
    for ip in found {
        if !seen.contains(&ip) {
            seen.push(ip);
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O resolvedor falso: `lento.*` demora 1,5 s, `dois.*` devolve o mesmo
    /// IPv4 duas vezes depois do IPv6, e o resto nao existe.
    fn fake(host: &str, _: u16) -> io::Result<Vec<IpAddr>> {
        if host.starts_with("lento.") {
            std::thread::sleep(Duration::from_millis(1500));
            return Ok(vec![IpAddr::from([127, 0, 0, 1])]);
        }
        if host.starts_with("dois.") {
            return Ok(vec![
                IpAddr::from([0, 0, 0, 0, 0, 0, 0, 1]),
                IpAddr::from([127, 0, 0, 1]),
                IpAddr::from([127, 0, 0, 1]),
            ]);
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Name or service not known",
        ))
    }

    /// Socket Unix, host vazio e IP literal nao passam pelo resolvedor.
    #[test]
    fn sockets_and_literals_skip_the_resolver() {
        let resolver = Resolver::new(fake, Duration::from_millis(100), 1);
        assert_eq!(
            resolver.addresses("/var/run/postgresql", 5432),
            Ok(Vec::new())
        );
        assert_eq!(resolver.addresses("", 5432), Ok(Vec::new()));
        assert_eq!(
            resolver.addresses("10.0.0.7", 5432),
            Ok(vec![IpAddr::from([10, 0, 0, 7])])
        );
        assert_eq!(resolver.pending(), 0);
    }

    /// O resolvedor que demora volta no prazo, com a frase; a vaga so' volta
    /// quando a thread termina, e a que passa do teto e' recusada na hora.
    #[test]
    fn a_hanging_resolver_returns_at_the_deadline_and_the_pending_are_capped() {
        let resolver = Resolver::new(fake, Duration::from_millis(200), 2);
        let start = std::time::Instant::now();
        let error = resolver.addresses("lento.exemplo", 5432).unwrap_err();
        assert!(error.contains("nao respondeu"), "{error}");
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "{:?}",
            start.elapsed()
        );
        assert!(resolver.addresses("lento.exemplo", 5432).is_err());
        assert_eq!(resolver.pending(), 2);
        let refused = resolver.addresses("lento.exemplo", 5432).unwrap_err();
        assert!(refused.contains("pendentes"), "{refused}");
        assert_eq!(resolver.pending(), 2, "a recusada nao ocupa vaga");
        std::thread::sleep(Duration::from_millis(1800));
        assert_eq!(resolver.pending(), 0);
    }

    /// A espera assincrona tambem volta no prazo, e a ordem do resolvedor
    /// fica, sem repetidos.
    #[test]
    fn the_async_wait_has_the_same_deadline_and_keeps_the_order() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let resolver = Resolver::new(fake, Duration::from_millis(200), 2);
        let start = std::time::Instant::now();
        let error = runtime
            .block_on(resolver.addresses_async("lento.exemplo", 5432))
            .unwrap_err();
        assert!(error.contains("nao respondeu"), "{error}");
        assert!(start.elapsed() < Duration::from_secs(1));
        let ordered = Resolver::new(fake, Duration::from_secs(1), 2);
        assert_eq!(
            runtime.block_on(ordered.addresses_async("dois.exemplo", 5432)),
            Ok(vec![
                IpAddr::from([0, 0, 0, 0, 0, 0, 0, 1]),
                IpAddr::from([127, 0, 0, 1])
            ])
        );
    }
}
