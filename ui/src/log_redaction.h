#pragma once

#include <QJsonObject>
#include <QString>

// O QUE NUNCA PODE APARECER NO LOG DO CLIENTE.
//
// POR QUE EXISTE (2026-09-04). O log registra os 200 primeiros bytes de cada
// pedido enviado ao core, e `appendErrorLog` grava em ARQUIVO. Quando o
// dominio `datasource` passou a poder mandar a senha da sessao em `params`,
// esse log virou o caminho mais curto para a senha sair do processo —
// exatamente a falha que o tipo `Secret` do core fecha do lado Rust
// (`DocsPublic/seguranca/40`). Redigir por NOME de campo fecha o caminho aqui.
//
// A lista e' de NOMES, nao de metodos, de proposito: um metodo novo que mande
// `password` ou `token` ja' nasce protegido, sem ninguem lembrar de
// acrescenta-lo a lugar nenhum.
//
// Saiu de dentro de `core_client_process.cpp` em 2026-09-26 para PODER SER
// MEDIDO: a §7.2 da `especificacoes/grafana-ui-ux-0.3.5.md` faz do teste de
// ausencia do token em log e persistencia um bloqueador de entrega, e uma
// funcao em namespace anonimo nao tem como ser testada.
//
// LIMITE DITO: isto protege VALOR DE CAMPO. Um segredo que viaje colado em
// texto livre — uma tecla digitada, um comando de terminal, uma mensagem de
// erro do servidor — nao tem nome de campo para redigir, e por isso os metodos
// que carregam texto bruto ficam fora do log inteiros, em vez de redigidos.
namespace kinein {

/// Verdadeiro para nomes de campo cujo VALOR nunca entra no log.
[[nodiscard]] bool isSecretField(const QString& key);

/// Copia o objeto trocando por `***` o valor de todo campo sensivel, em
/// qualquer profundidade — objetos dentro de arrays inclusive.
[[nodiscard]] QJsonObject redactSecrets(const QJsonObject& request);

} // namespace kinein
