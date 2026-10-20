//! One permission assessment for discovery, recovery and operation guards.
use std::path::Path;

use kinein_protocol::{SerialAccess, SerialAccessState};

#[cfg(windows)]
pub(super) fn measure(device: &Path, _: &Path, _: &Path, _: &Path) -> SerialAccess {
    SerialAccess {
        readable_writable: false,
        state: SerialAccessState::Unsupported,
        mode: "?".to_owned(),
        group: None,
        hint: Some(format!(
            "{}: a porta serial ainda não é suportada no Windows.",
            device.display()
        )),
    }
}

#[cfg(unix)]
pub(super) use unix::measure;

// Keep Unix credentials and metadata together; Windows has its own stub above.
#[cfg(unix)]
mod unix {
    use super::{Path, SerialAccess, SerialAccessState};
    pub(in crate::serial) fn measure(
        device: &Path,
        proc_root: &Path,
        accounts_file: &Path,
        groups: &Path,
    ) -> SerialAccess {
        use std::os::unix::fs::MetadataExt;
        let metadata = match std::fs::metadata(device) {
            Ok(metadata) => metadata,
            Err(error) => {
                return SerialAccess {
                    readable_writable: false,
                    state: if error.kind() == std::io::ErrorKind::NotFound {
                        SerialAccessState::Unavailable
                    } else {
                        SerialAccessState::PermissionDenied
                    },
                    mode: "?".to_owned(),
                    group: None,
                    hint: Some(format!(
                        "{}: não foi possível consultar a porta ({error}). Confira a conexão e o acesso; depois atualize os dispositivos.",
                        device.display()
                    )),
                };
            }
        };
        let access = rustix::fs::access(
            device,
            rustix::fs::Access::READ_OK | rustix::fs::Access::WRITE_OK,
        );
        let group_text = std::fs::read_to_string(groups).unwrap_or_default();
        let group = group_text.lines().find_map(|line| {
            let fields: Vec<_> = line.split(':').collect();
            (fields.len() == 4 && fields[2].parse::<u32>().ok() == Some(metadata.gid()))
                .then(|| (fields[0].to_owned(), fields[3].to_owned()))
        });
        let session = Session::load(proc_root, accounts_file);
        let state = if access.is_ok() {
            SerialAccessState::Ready
        } else if access == Err(rustix::io::Errno::NOENT) {
            SerialAccessState::Unavailable
        } else if access == Err(rustix::io::Errno::ACCESS) || access == Err(rustix::io::Errno::PERM)
        {
            denial_state(
                metadata.uid(),
                metadata.gid(),
                metadata.mode(),
                &session,
                group.as_ref(),
            )
        } else {
            SerialAccessState::PermissionDenied
        };
        let mode = symbolic_mode(metadata.mode());
        let group_name = group.map(|(name, _)| name);
        let hint = (state != SerialAccessState::Ready).then(|| {
        let name = group_name.as_deref().unwrap_or("desconhecido");
        match state {
            SerialAccessState::SessionRefreshRequired => format!("Sua conta já está cadastrada em `{name}`, mas esta sessão ainda não recebeu o grupo. Salve seu trabalho, saia e entre na sessão do sistema; depois reabra a IDE e verifique o acesso."),
            SerialAccessState::GroupRequired => format!("Esta sessão não tem acesso à porta e sua conta local ainda não está cadastrada em `{name}`. Abra Preparar acesso para cadastrar o grupo e atualizar a sessão."),
            SerialAccessState::Unavailable => "A porta deixou de estar disponível. Reconecte a placa e atualize os dispositivos.".to_owned(),
            _ => format!("{}: leitura e escrita negadas ({mode}, grupo `{name}`). Não foi comprovado que adicionar um grupo resolva. Abra Preparar acesso para conferir as permissões.", device.display()),
        }
    });
        SerialAccess {
            readable_writable: access.is_ok(),
            state,
            mode,
            group: group_name,
            hint,
        }
    }

    #[derive(Default)]
    struct Session {
        uid: Option<u32>,
        groups: Option<Vec<u32>>,
        account: Option<(String, u32)>,
    }

    impl Session {
        fn load(proc_root: &Path, accounts_file: &Path) -> Self {
            let status = std::fs::read_to_string(proc_root.join("self/status")).unwrap_or_default();
            let values = |key: &str| -> Option<Vec<u32>> {
                status
                    .lines()
                    .find_map(|line| line.strip_prefix(key))?
                    .split_whitespace()
                    .map(str::parse)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()
            };
            let uid = values("Uid:").and_then(|ids| ids.first().copied());
            let primary = values("Gid:").and_then(|ids| ids.first().copied());
            let groups = values("Groups:").zip(primary).map(|(mut groups, primary)| {
                groups.push(primary);
                groups
            });
            let account = std::fs::read_to_string(accounts_file)
                .ok()
                .and_then(|text| {
                    text.lines().find_map(|line| {
                        let fields: Vec<_> = line.split(':').collect();
                        if fields.len() != 7
                            || uid.is_none()
                            || fields[2].parse::<u32>().ok() != uid
                        {
                            return None;
                        }
                        Some((fields[0].to_owned(), fields[3].parse().ok()?))
                    })
                });
            Self {
                uid,
                groups,
                account,
            }
        }
    }

    fn denial_state(
        owner: u32,
        gid: u32,
        mode: u32,
        session: &Session,
        group: Option<&(String, String)>,
    ) -> SerialAccessState {
        use SerialAccessState::{GroupRequired, PermissionDenied, SessionRefreshRequired};
        let (Some(uid), Some(active)) = (session.uid, &session.groups) else {
            return PermissionDenied;
        };
        // Owner bits take precedence. Membership cannot repair them, nor a denial
        // when the owning group already participates in access(2).
        if owner == uid || active.contains(&gid) || mode & 0o060 != 0o060 {
            return PermissionDenied;
        }
        let (Some((username, primary)), Some((name, members))) = (&session.account, group) else {
            return PermissionDenied;
        };
        if *primary == gid || members.split(',').any(|member| member == username) {
            return SessionRefreshRequired;
        }
        if matches!(name.as_str(), "dialout" | "uucp" | "plugdev" | "tty") {
            GroupRequired
        } else {
            PermissionDenied
        }
    }

    /// Symbolic mode for evidence; never used as the access verdict.
    fn symbolic_mode(mode: u32) -> String {
        let kind = match mode & 0o170_000 {
            0o020_000 => 'c',
            0o060_000 => 'b',
            0o120_000 => 'l',
            0o040_000 => 'd',
            _ => '-',
        };
        let mut text = String::with_capacity(10);
        text.push(kind);
        for shift in [6u32, 3, 0] {
            let bits = (mode >> shift) & 0o7;
            text.push(if bits & 4 != 0 { 'r' } else { '-' });
            text.push(if bits & 2 != 0 { 'w' } else { '-' });
            text.push(if bits & 1 != 0 { 'x' } else { '-' });
        }
        text
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn denied_access_distinguishes_registration_session_and_other_causes() {
            use SerialAccessState::{GroupRequired, PermissionDenied, SessionRefreshRequired};
            assert_eq!(symbolic_mode(0o020_660), "crw-rw----");
            assert_eq!(symbolic_mode(0o100_755), "-rwxr-xr-x");
            let mut session = Session {
                uid: Some(1000),
                groups: Some(vec![1000]),
                account: Some(("alice".to_owned(), 1000)),
            };
            let registered = ("dialout".to_owned(), "alice,bob".to_owned());
            let unregistered = ("dialout".to_owned(), "alice2,bob".to_owned());
            assert_eq!(
                denial_state(0, 20, 0o660, &session, Some(&registered)),
                SessionRefreshRequired
            );
            assert_eq!(
                denial_state(0, 20, 0o660, &session, Some(&unregistered)),
                GroupRequired
            );
            // Group names, usernames and membership are exact, not substring matches.
            assert_eq!(
                denial_state(
                    0,
                    20,
                    0o660,
                    &session,
                    Some(&("custom".to_owned(), "bob".to_owned()))
                ),
                PermissionDenied
            );
            for (owner, mode) in [(1000, 0o060), (0, 0o600), (0, 0o640)] {
                assert_eq!(
                    denial_state(owner, 20, mode, &session, Some(&registered)),
                    PermissionDenied
                );
            }
            session.groups.as_mut().unwrap().push(20);
            assert_eq!(
                denial_state(0, 20, 0o660, &session, Some(&registered)),
                PermissionDenied
            );
            session.groups = Some(vec![1000]);
            session.account.as_mut().unwrap().1 = 20;
            assert_eq!(
                denial_state(0, 20, 0o660, &session, Some(&unregistered)),
                SessionRefreshRequired
            );
            session.account = None;
            assert_eq!(
                denial_state(0, 20, 0o660, &session, Some(&registered)),
                PermissionDenied
            );
            session.groups = None;
            assert_eq!(
                denial_state(0, 20, 0o660, &session, Some(&registered)),
                PermissionDenied
            );
            assert_eq!(
                denial_state(0, 20, 0o660, &Session::default(), None),
                PermissionDenied
            );
        }

        #[test]
        fn process_identity_includes_primary_group_and_does_not_depend_on_user_env() {
            let root =
                std::env::temp_dir().join(format!("kinein-serial-identity-{}", std::process::id()));
            std::fs::create_dir_all(root.join("self")).unwrap();
            std::fs::write(
                root.join("self/status"),
                "Uid:\t1000 1000 1000 1000\nGid:\t20 20 20 20\nGroups:\t1000 27\n",
            )
            .unwrap();
            std::fs::write(
                root.join("accounts_file"),
                "other:x:1001:20::/:/bin/sh\nalice:x:1000:1000::/:/bin/sh\n",
            )
            .unwrap();
            let session = Session::load(&root, &root.join("accounts_file"));
            assert_eq!(session.uid, Some(1000));
            assert_eq!(session.groups, Some(vec![1000, 27, 20]));
            assert_eq!(session.account, Some(("alice".to_owned(), 1000)));
            let group = ("dialout".to_owned(), "alice".to_owned());
            assert_eq!(
                denial_state(0, 20, 0o660, &session, Some(&group)),
                SerialAccessState::PermissionDenied
            );
            std::fs::write(
                root.join("self/status"),
                "Uid:\t1000\nGid:\t1000\nGroups:\tbroken\n",
            )
            .unwrap();
            assert!(
                Session::load(&root, &root.join("accounts_file"))
                    .groups
                    .is_none()
            );
            std::fs::remove_file(root.join("accounts_file")).unwrap();
            assert!(
                Session::load(&root, &root.join("accounts_file"))
                    .account
                    .is_none()
            );
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
