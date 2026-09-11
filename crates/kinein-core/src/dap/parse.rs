//! Interpretar o que o adapter respondeu: breakpoints, frames e variaveis.
//!
//! Separado em 2026-09-03 (etapa 15 do `roadmaps/34`). E' o unico arquivo do
//! `dap/` que se testa sem subir processo nenhum — os testes no fim do arquivo
//! sao a razao de ele existir separado do transporte e da sessao.

use kinein_protocol::{
    BreakpointInfo, DebugEvaluateResult, SourceBreakpointParams, StackFrameInfo, VariableInfo,
};
use serde_json::{Value, json};

/// Monta os `arguments` de `setBreakpoints` para um arquivo.
///
/// `condition` e `hitCondition` so' entram no JSON quando existem: um campo
/// nulo nao e' a mesma coisa que campo ausente para um adapter, e mandar
/// `"condition": null` e' pedir para um deles tratar como expressao vazia.
pub(super) fn breakpoints_arguments(file: &str, breakpoints: &[SourceBreakpointParams]) -> Value {
    json!({
        "source": { "path": file },
        "breakpoints": breakpoints
            .iter()
            .map(|bp| {
                let mut item = json!({ "line": bp.line });
                if let Some(condition) = bp.condition.as_deref().filter(|c| !c.trim().is_empty()) {
                    item["condition"] = json!(condition);
                }
                if let Some(hit) = bp.hit_condition.as_deref().filter(|c| !c.trim().is_empty()) {
                    item["hitCondition"] = json!(hit);
                }
                item
            })
            .collect::<Vec<_>>(),
    })
}

/// Converte o `body` de `evaluate` no resultado do protocolo.
///
/// O DAP manda `result` obrigatorio, `type` opcional e `variablesReference`
/// obrigatorio — `> 0` quando o valor tem filhos, que e' o mesmo contrato do
/// `reference` que `debug.variables` ja' usa.
pub(super) fn parse_evaluate(expression: &str, body: &Value) -> DebugEvaluateResult {
    DebugEvaluateResult {
        expression: expression.to_string(),
        result: body
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        type_name: body.get("type").and_then(Value::as_str).map(str::to_string),
        reference: body
            .get("variablesReference")
            .and_then(Value::as_i64)
            .unwrap_or(0),
    }
}

/// Converte o `body` de `setBreakpoints` no resultado do protocolo.
pub(super) fn parse_breakpoints(body: &Value, requested: &[u32]) -> Vec<BreakpointInfo> {
    let Some(items) = body.get("breakpoints").and_then(Value::as_array) else {
        return requested
            .iter()
            .map(|&line| BreakpointInfo {
                line,
                verified: false,
            })
            .collect();
    };
    items
        .iter()
        .enumerate()
        .map(|(index, item)| BreakpointInfo {
            line: item
                .get("line")
                .and_then(Value::as_u64)
                .and_then(|line| u32::try_from(line).ok())
                .or_else(|| requested.get(index).copied())
                .unwrap_or(0),
            verified: item
                .get("verified")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
        .collect()
}

/// Converte o `body` de `stackTrace` nos frames do protocolo.
pub(super) fn parse_stack_frames(body: &Value) -> Vec<StackFrameInfo> {
    let Some(items) = body.get("stackFrames").and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .map(|frame| StackFrameInfo {
            id: frame.get("id").and_then(Value::as_i64).unwrap_or(0),
            name: frame
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_owned(),
            file: frame
                .get("source")
                .and_then(|source| source.get("path"))
                .and_then(Value::as_str)
                .map(str::to_owned),
            line: frame
                .get("line")
                .and_then(Value::as_u64)
                .and_then(|line| u32::try_from(line).ok())
                .filter(|&line| line > 0),
        })
        .collect()
}

/// Acha o `variablesReference` do primeiro escopo nao-caro (Locals).
pub(super) fn preferred_scope_reference(body: &Value) -> Option<i64> {
    let scopes: Vec<&Value> = body
        .get("scopes")
        .and_then(Value::as_array)?
        .iter()
        .filter(|scope| scope.get("expensive").and_then(Value::as_bool) != Some(true))
        .collect();
    // O lldb-dap lista `Locals` primeiro; o GDB lista `Registers` primeiro e
    // `Globals` depois (medido em 2026-09-11 contra o gdb 17.2 num alvo
    // bare-metal). "O primeiro escopo barato" mostrava r0..r15 onde o usuario
    // esperava a variavel dele. Registradores so' quando nao ha' mais nada.
    let is_registers = |scope: &&Value| {
        scope.get("presentationHint").and_then(Value::as_str) == Some("registers")
            || scope.get("name").and_then(Value::as_str) == Some("Registers")
    };
    scopes
        .iter()
        .find(|scope| !is_registers(scope))
        .or_else(|| scopes.first())
        .and_then(|scope| scope.get("variablesReference"))
        .and_then(Value::as_i64)
        .filter(|&reference| reference > 0)
}

/// Converte o `body` de `variables` nas variaveis do protocolo.
pub(super) fn parse_variables(body: &Value) -> Vec<VariableInfo> {
    let Some(items) = body.get("variables").and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .map(|variable| VariableInfo {
            name: variable
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_owned(),
            value: variable
                .get("value")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            type_name: variable
                .get("type")
                .and_then(Value::as_str)
                .map(str::to_owned),
            reference: variable
                .get("variablesReference")
                .and_then(Value::as_i64)
                .unwrap_or(0),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use kinein_protocol::SourceBreakpointParams;

    use super::{
        breakpoints_arguments, parse_evaluate, parse_stack_frames, parse_variables,
        preferred_scope_reference,
    };

    #[test]
    fn stack_frames_map_source_and_drop_zero_lines() {
        let frames = parse_stack_frames(&json!({
            "stackFrames": [
                { "id": 4, "name": "soma", "line": 4,
                  "source": { "path": "/w/main.c" } },
                { "id": 5, "name": "??", "line": 0 },
            ]
        }));
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].id, 4);
        assert_eq!(frames[0].file.as_deref(), Some("/w/main.c"));
        assert_eq!(frames[0].line, Some(4));
        assert_eq!(frames[1].file, None);
        assert_eq!(frames[1].line, None);
    }

    #[test]
    fn scope_selection_skips_expensive_and_requires_reference() {
        let reference = preferred_scope_reference(&json!({
            "scopes": [
                { "name": "Registers", "expensive": true,
                  "variablesReference": 9 },
                { "name": "Locals", "variablesReference": 3 },
            ]
        }));
        assert_eq!(reference, Some(3));
        assert_eq!(preferred_scope_reference(&json!({ "scopes": [] })), None);
    }

    /// O GDB lista `Registers` PRIMEIRO e barato (medido em 2026-09-11 num
    /// alvo bare-metal: `[Registers, Globals]`). O primeiro escopo barato
    /// mostraria r0..r15 no lugar da variavel do usuario.
    #[test]
    fn registers_come_last_even_when_the_adapter_lists_them_first() {
        let gdb = json!({
            "scopes": [
                { "name": "Registers", "presentationHint": "registers",
                  "expensive": false, "variablesReference": 1 },
                { "name": "Globals", "expensive": false, "variablesReference": 2 },
            ]
        });
        assert_eq!(preferred_scope_reference(&gdb), Some(2));
        // So' registradores: melhor eles do que nada.
        let so_registradores = json!({
            "scopes": [{ "name": "Registers", "presentationHint": "registers",
                         "variablesReference": 1 }]
        });
        assert_eq!(preferred_scope_reference(&so_registradores), Some(1));
    }

    #[test]
    fn variables_carry_type_and_expansion_reference() {
        let variables = parse_variables(&json!({
            "variables": [
                { "name": "a", "value": "2", "type": "int",
                  "variablesReference": 0 },
                { "name": "p", "value": "{...}", "variablesReference": 12 },
            ]
        }));
        assert_eq!(variables[0].type_name.as_deref(), Some("int"));
        assert_eq!(variables[0].reference, 0);
        assert_eq!(variables[1].reference, 12);
        assert_eq!(variables[1].type_name, None);
    }

    #[test]
    fn evaluate_body_maps_result_type_and_reference() {
        let com_tipo = parse_evaluate(
            "conta",
            &json!({ "result": "42", "type": "int", "variablesReference": 0 }),
        );
        assert_eq!(com_tipo.expression, "conta");
        assert_eq!(com_tipo.result, "42");
        assert_eq!(com_tipo.type_name.as_deref(), Some("int"));
        assert_eq!(com_tipo.reference, 0);

        // Sem `type` e com filhos: o reference > 0 e o que deixa a UI expandir,
        // mesmo contrato do `debug.variables`.
        let expansivel =
            parse_evaluate("no", &json!({ "result": "{...}", "variablesReference": 7 }));
        assert_eq!(expansivel.type_name, None);
        assert_eq!(expansivel.reference, 7);

        // Corpo torto nao entra em panico: o watch mostra vazio.
        let vazio = parse_evaluate("x", &json!({}));
        assert_eq!(vazio.result, "");
        assert_eq!(vazio.reference, 0);
    }

    #[test]
    fn breakpoint_arguments_omit_empty_conditions() {
        let args = breakpoints_arguments(
            "/w/main.c",
            &[
                SourceBreakpointParams {
                    line: 3,
                    condition: Some("  ".into()),
                    hit_condition: None,
                },
                SourceBreakpointParams {
                    line: 9,
                    condition: Some("i == 42".into()),
                    hit_condition: Some("5".into()),
                },
            ],
        );
        let items = args["breakpoints"].as_array().unwrap();

        // Condicao so' de espacos e' o mesmo que nao ter condicao: mandar
        // `"condition": "  "` faria o adapter avaliar uma expressao invalida e
        // o breakpoint nunca pararia.
        assert!(
            items[0].get("condition").is_none(),
            "espacos viraram condicao"
        );
        assert!(items[0].get("hitCondition").is_none());

        assert_eq!(items[1]["condition"], "i == 42");
        assert_eq!(items[1]["hitCondition"], "5");
    }
}
