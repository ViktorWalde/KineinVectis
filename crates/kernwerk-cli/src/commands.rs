//! CLI command dispatch: turns argv into a single JSON-RPC request on stdout.

use std::io::Write;

use kernwerk_protocol::JsonRpcRequest;
use serde_json::json;

use crate::error::CliError;

/// Parses argv and writes one JSON-RPC request, or a usage error.
pub fn run<I, W, E>(args: I, mut stdout: W, mut stderr: E) -> Result<(), CliError>
where
    I: IntoIterator<Item = String>,
    W: Write,
    E: Write,
{
    let args = args.into_iter().collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        None | Some("ping") => write_request(
            &mut stdout,
            &JsonRpcRequest::new(1_i64, "core.ping", Some(json!({}))),
        ),
        Some("list-commands") => write_request(
            &mut stdout,
            &JsonRpcRequest::new(1_i64, "command.list", Some(json!({}))),
        ),
        Some("shutdown") => write_request(
            &mut stdout,
            &JsonRpcRequest::new(1_i64, "core.shutdown", Some(json!({}))),
        ),
        Some("tools") => run_tools_command(&args, &mut stdout, &mut stderr),
        Some("workspace") => run_workspace_command(&args, &mut stdout, &mut stderr),
        Some("fs") => run_fs_command(&args, &mut stdout, &mut stderr),
        Some("build") => write_request(
            &mut stdout,
            &JsonRpcRequest::new(1_i64, "build.run", Some(json!({}))),
        ),
        Some("test") => {
            let params = args
                .get(1)
                .map_or_else(|| json!({}), |filter| json!({ "filter": filter }));
            write_request(
                &mut stdout,
                &JsonRpcRequest::new(1_i64, "test.run", Some(params)),
            )
        }
        Some("quality") => write_request(
            &mut stdout,
            &JsonRpcRequest::new(1_i64, "quality.run", Some(json!({}))),
        ),
        Some("--help" | "-h") => write_help(&mut stdout),
        Some(command) => {
            writeln!(stderr, "unknown command: {command}").map_err(CliError::Write)?;
            write_help(&mut stderr)?;
            Err(CliError::UnknownCommand(command.to_owned()))
        }
    }
}

fn run_tools_command<W, E>(args: &[String], stdout: &mut W, stderr: &mut E) -> Result<(), CliError>
where
    W: Write,
    E: Write,
{
    match args.get(1).map(String::as_str) {
        Some("detect") => write_request(
            stdout,
            &JsonRpcRequest::new(1_i64, "tools.detect", Some(json!({}))),
        ),
        Some("status") => write_request(
            stdout,
            &JsonRpcRequest::new(1_i64, "tools.status", Some(json!({}))),
        ),
        subcommand => {
            let subcommand = subcommand.unwrap_or("<vazio>");
            writeln!(stderr, "unknown tools subcommand: {subcommand}").map_err(CliError::Write)?;
            write_help(stderr)?;
            Err(CliError::UnknownCommand(format!("tools {subcommand}")))
        }
    }
}

fn run_workspace_command<W, E>(
    args: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), CliError>
where
    W: Write,
    E: Write,
{
    match args.get(1).map(String::as_str) {
        Some("open") => write_workspace_path_request(args, stdout, stderr, "open"),
        Some("browse") => write_workspace_path_request(args, stdout, stderr, "browse"),
        Some("mkdir") => write_workspace_create_folder_request(args, stdout, stderr),
        Some("new") => write_workspace_create_project_request(args, stdout, stderr),
        Some("status") => write_request(
            stdout,
            &JsonRpcRequest::new(1_i64, "workspace.status", Some(json!({}))),
        ),
        Some("close") => write_request(
            stdout,
            &JsonRpcRequest::new(1_i64, "workspace.close", Some(json!({}))),
        ),
        subcommand => {
            let subcommand = subcommand.unwrap_or("<vazio>");
            writeln!(stderr, "unknown workspace subcommand: {subcommand}")
                .map_err(CliError::Write)?;
            write_help(stderr)?;
            Err(CliError::UnknownCommand(format!("workspace {subcommand}")))
        }
    }
}

fn write_workspace_path_request<W, E>(
    args: &[String],
    stdout: &mut W,
    stderr: &mut E,
    subcommand: &'static str,
) -> Result<(), CliError>
where
    W: Write,
    E: Write,
{
    let Some(path) = args.get(2) else {
        writeln!(stderr, "workspace {subcommand} requires a path").map_err(CliError::Write)?;
        write_help(stderr)?;
        return Err(CliError::MissingArgument(match subcommand {
            "open" => "workspace open <path>",
            "browse" => "workspace browse <path>",
            _ => "workspace <command> <path>",
        }));
    };
    write_request(
        stdout,
        &JsonRpcRequest::new(
            1_i64,
            format!("workspace.{subcommand}"),
            Some(json!({ "path": path })),
        ),
    )
}

fn write_workspace_create_folder_request<W, E>(
    args: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), CliError>
where
    W: Write,
    E: Write,
{
    let (Some(parent), Some(name)) = (args.get(2), args.get(3)) else {
        writeln!(stderr, "workspace mkdir requires parent and name").map_err(CliError::Write)?;
        write_help(stderr)?;
        return Err(CliError::MissingArgument("workspace mkdir <parent> <name>"));
    };
    write_request(
        stdout,
        &JsonRpcRequest::new(
            1_i64,
            "workspace.createFolder",
            Some(json!({ "parent": parent, "name": name })),
        ),
    )
}

fn write_workspace_create_project_request<W, E>(
    args: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), CliError>
where
    W: Write,
    E: Write,
{
    let (Some(parent), Some(name), Some(template)) = (args.get(2), args.get(3), args.get(4)) else {
        writeln!(stderr, "workspace new requires parent, name and template")
            .map_err(CliError::Write)?;
        write_help(stderr)?;
        return Err(CliError::MissingArgument(
            "workspace new <parent> <name> <empty|cppCmake|rustCargo>",
        ));
    };
    write_request(
        stdout,
        &JsonRpcRequest::new(
            1_i64,
            "workspace.createProject",
            Some(json!({ "parent": parent, "name": name, "template": template })),
        ),
    )
}

fn run_fs_command<W, E>(args: &[String], stdout: &mut W, stderr: &mut E) -> Result<(), CliError>
where
    W: Write,
    E: Write,
{
    match (args.get(1).map(String::as_str), args.get(2)) {
        (Some("list"), Some(path)) => write_request(
            stdout,
            &JsonRpcRequest::new(1_i64, "fs.list", Some(json!({ "path": path }))),
        ),
        (Some("read"), Some(path)) => write_request(
            stdout,
            &JsonRpcRequest::new(1_i64, "fs.read", Some(json!({ "path": path }))),
        ),
        (Some("search"), Some(query)) => write_request(
            stdout,
            &JsonRpcRequest::new(1_i64, "fs.search", Some(json!({ "query": query }))),
        ),
        (Some("delete"), Some(path)) => write_request(
            stdout,
            &JsonRpcRequest::new(1_i64, "fs.delete", Some(json!({ "path": path }))),
        ),
        (Some("rename"), Some(from)) => {
            if let Some(to) = args.get(3) {
                write_request(
                    stdout,
                    &JsonRpcRequest::new(
                        1_i64,
                        "fs.rename",
                        Some(json!({ "from": from, "to": to })),
                    ),
                )
            } else {
                writeln!(stderr, "fs rename precisa de <from> e <to>").map_err(CliError::Write)?;
                write_help(stderr)?;
                Err(CliError::MissingArgument("fs rename <from> <to>"))
            }
        }
        (Some("list" | "read" | "search" | "delete" | "rename"), None) => {
            writeln!(stderr, "fs precisa de um argumento").map_err(CliError::Write)?;
            write_help(stderr)?;
            Err(CliError::MissingArgument(
                "fs list|read <path> | fs search <query> | fs rename <from> <to> | fs delete <path>",
            ))
        }
        (subcommand, _) => {
            let subcommand = subcommand.unwrap_or("<vazio>");
            writeln!(stderr, "unknown fs subcommand: {subcommand}").map_err(CliError::Write)?;
            write_help(stderr)?;
            Err(CliError::UnknownCommand(format!("fs {subcommand}")))
        }
    }
}

fn write_request<W>(writer: &mut W, request: &JsonRpcRequest) -> Result<(), CliError>
where
    W: Write,
{
    serde_json::to_writer(&mut *writer, request).map_err(CliError::Serialize)?;
    writer.write_all(b"\n").map_err(CliError::Write)
}

fn write_help<W>(writer: &mut W) -> Result<(), CliError>
where
    W: Write,
{
    writer
        .write_all(
            b"Usage: kernwerk-cli <command>\n\
              Commands:\n\
              \x20 ping\n\
              \x20 list-commands\n\
              \x20 shutdown\n\
              \x20 tools detect|status\n\
              \x20 workspace open <path>\n\
              \x20 workspace browse <path>\n\
              \x20 workspace mkdir <parent> <name>\n\
              \x20 workspace new <parent> <name> <empty|cppCmake|rustCargo>\n\
              \x20 workspace status\n\
              \x20 workspace close\n\
              \x20 fs list <path>\n\
              \x20 fs read <path>\n\
              \x20 fs search <query>\n\
              \x20 fs rename <from> <to>\n\
              \x20 fs delete <path>\n\
              \x20 build\n\
              \x20 test [filtro]\n\
              \x20 quality\n\
              Emits one JSON-RPC request to stdout.\n",
        )
        .map_err(CliError::Write)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::run;
    use crate::error::CliError;

    #[test]
    fn ping_command_writes_json_rpc_request() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        run(["ping".to_owned()], &mut stdout, &mut stderr).unwrap();

        let request = serde_json::from_slice::<Value>(&stdout).unwrap();
        assert_eq!(request["method"], "core.ping");
        assert!(stderr.is_empty());
    }

    #[test]
    fn tools_detect_command_writes_json_rpc_request() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        run(
            ["tools".to_owned(), "detect".to_owned()],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        let request = serde_json::from_slice::<Value>(&stdout).unwrap();
        assert_eq!(request["method"], "tools.detect");
        assert!(stderr.is_empty());
    }

    #[test]
    fn tools_without_subcommand_fails() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let error = run(["tools".to_owned()], &mut stdout, &mut stderr).unwrap_err();

        assert!(error.to_string().contains("tools"));
        assert!(stdout.is_empty());
        assert!(!stderr.is_empty());
    }

    #[test]
    fn workspace_open_command_includes_path_param() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        run(
            [
                "workspace".to_owned(),
                "open".to_owned(),
                "/home/user/projeto".to_owned(),
            ],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        let request = serde_json::from_slice::<Value>(&stdout).unwrap();
        assert_eq!(request["method"], "workspace.open");
        assert_eq!(request["params"]["path"], "/home/user/projeto");
        assert!(stderr.is_empty());
    }

    #[test]
    fn workspace_open_without_path_fails() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let error = run(
            ["workspace".to_owned(), "open".to_owned()],
            &mut stdout,
            &mut stderr,
        )
        .unwrap_err();

        assert!(error.to_string().contains("workspace open <path>"));
        assert!(stdout.is_empty());
    }

    #[test]
    fn workspace_browse_command_includes_path_param() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        run(
            [
                "workspace".to_owned(),
                "browse".to_owned(),
                "/home/user".to_owned(),
            ],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        let request = serde_json::from_slice::<Value>(&stdout).unwrap();
        assert_eq!(request["method"], "workspace.browse");
        assert_eq!(request["params"]["path"], "/home/user");
        assert!(stderr.is_empty());
    }

    #[test]
    fn workspace_browse_without_path_fails() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let error = run(
            ["workspace".to_owned(), "browse".to_owned()],
            &mut stdout,
            &mut stderr,
        )
        .unwrap_err();

        assert!(error.to_string().contains("workspace browse <path>"));
        assert!(stdout.is_empty());
    }

    #[test]
    fn workspace_mkdir_command_includes_parent_and_name() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        run(
            [
                "workspace".to_owned(),
                "mkdir".to_owned(),
                "/home/user".to_owned(),
                "modulo".to_owned(),
            ],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        let request = serde_json::from_slice::<Value>(&stdout).unwrap();
        assert_eq!(request["method"], "workspace.createFolder");
        assert_eq!(request["params"]["parent"], "/home/user");
        assert_eq!(request["params"]["name"], "modulo");
        assert!(stderr.is_empty());
    }

    #[test]
    fn workspace_new_command_includes_project_template() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        run(
            [
                "workspace".to_owned(),
                "new".to_owned(),
                "/home/user".to_owned(),
                "demo".to_owned(),
                "cppCmake".to_owned(),
            ],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        let request = serde_json::from_slice::<Value>(&stdout).unwrap();
        assert_eq!(request["method"], "workspace.createProject");
        assert_eq!(request["params"]["parent"], "/home/user");
        assert_eq!(request["params"]["name"], "demo");
        assert_eq!(request["params"]["template"], "cppCmake");
        assert!(stderr.is_empty());
    }

    #[test]
    fn fs_list_command_includes_path_param() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        run(
            ["fs".to_owned(), "list".to_owned(), "/x".to_owned()],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        let request = serde_json::from_slice::<Value>(&stdout).unwrap();
        assert_eq!(request["method"], "fs.list");
        assert_eq!(request["params"]["path"], "/x");
    }

    #[test]
    fn fs_rename_command_includes_from_and_to() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        run(
            [
                "fs".to_owned(),
                "rename".to_owned(),
                "/a.rs".to_owned(),
                "/b.rs".to_owned(),
            ],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        let request = serde_json::from_slice::<Value>(&stdout).unwrap();
        assert_eq!(request["method"], "fs.rename");
        assert_eq!(request["params"]["from"], "/a.rs");
        assert_eq!(request["params"]["to"], "/b.rs");
    }

    #[test]
    fn fs_rename_without_destination_fails() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let error = run(
            ["fs".to_owned(), "rename".to_owned(), "/a.rs".to_owned()],
            &mut stdout,
            &mut stderr,
        )
        .unwrap_err();

        assert!(matches!(error, CliError::MissingArgument(_)));
        assert!(stdout.is_empty());
        assert!(!stderr.is_empty());
    }

    #[test]
    fn fs_delete_command_includes_path_param() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        run(
            ["fs".to_owned(), "delete".to_owned(), "/x.rs".to_owned()],
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        let request = serde_json::from_slice::<Value>(&stdout).unwrap();
        assert_eq!(request["method"], "fs.delete");
        assert_eq!(request["params"]["path"], "/x.rs");
    }

    #[test]
    fn unknown_command_fails() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let error = run(["unknown".to_owned()], &mut stdout, &mut stderr).unwrap_err();

        assert_eq!(error.to_string(), "unknown command: unknown");
        assert!(stdout.is_empty());
        assert!(!stderr.is_empty());
    }
}
