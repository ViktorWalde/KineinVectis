// O QUARTO teste C++ do projeto (2026-09-26), e ele existe porque uma
// especificacao mandou: a §7.2 da `especificacoes/grafana-ui-ux-0.3.5.md`
// lista "testes de ciclo de vida e de ausencia em persistencia/log" como
// BLOQUEADORES da entrega do token de sessao do Grafana.
//
// A redacao ja' existia desde 2026-09-04, quando a senha de banco passou a
// viajar em `params`. O que nao existia era PROVA: ela morava num namespace
// anonimo dentro do `core_client_process.cpp`, onde nenhum teste alcanca. Uma
// protecao que ninguem mede e' uma protecao que ninguem sabe se ainda esta'
// la' depois do proximo refactor.
#include "log_redaction.h"

#include <QJsonArray>
#include <QJsonDocument>
#include <QtTest>

using kinein::isSecretField;
using kinein::redactSecrets;

namespace {

/// O pedido como o `sendRequest` o monta, para o teste falar a mesma lingua
/// que o log.
QJsonObject pedido(const QString& metodo, const QJsonObject& params)
{
    return QJsonObject{
        {QStringLiteral("jsonrpc"), QStringLiteral("2.0")},
        {QStringLiteral("id"), 7},
        {QStringLiteral("method"), metodo},
        {QStringLiteral("params"), params},
    };
}

QString comoTexto(const QJsonObject& objeto)
{
    return QString::fromUtf8(QJsonDocument(objeto).toJson(QJsonDocument::Compact));
}

} // namespace

class TestLogRedaction : public QObject
{
    Q_OBJECT

private slots:
    void grafana_probe_token_never_reaches_the_log();
    void datasource_password_never_reaches_the_log();
    void field_names_are_matched_without_case();
    void nested_objects_and_arrays_are_reached();
    void absent_token_stays_absent();
    void ordinary_fields_are_preserved();
};

// O CASO QUE A §7.2 EXIGE: o token do Grafana viaja em `params` de
// `grafana.probe`, e o log grava em arquivo.
void TestLogRedaction::grafana_probe_token_never_reaches_the_log()
{
    const QString segredo = QStringLiteral("glsa_umTokenDeServicoQualquer");
    const QJsonObject limpo = redactSecrets(pedido(QStringLiteral("grafana.probe"),
                                                   QJsonObject{
                                                       {QStringLiteral("token"), segredo},
                                                   }));
    QVERIFY(!comoTexto(limpo).contains(segredo));
    QCOMPARE(limpo.value(QStringLiteral("params"))
                 .toObject()
                 .value(QStringLiteral("token"))
                 .toString(),
             QStringLiteral("***"));
    // O metodo continua legivel: redigir o log nao pode cegar quem depura.
    QCOMPARE(limpo.value(QStringLiteral("method")).toString(), QStringLiteral("grafana.probe"));
}

void TestLogRedaction::datasource_password_never_reaches_the_log()
{
    const QString segredo = QStringLiteral("senha-da-sessao");
    const QJsonObject limpo =
        redactSecrets(pedido(QStringLiteral("datasource.connect"),
                             QJsonObject{
                                 {QStringLiteral("password"), segredo},
                                 {QStringLiteral("host"), QStringLiteral("localhost")},
                             }));
    QVERIFY(!comoTexto(limpo).contains(segredo));
    QCOMPARE(limpo.value(QStringLiteral("params"))
                 .toObject()
                 .value(QStringLiteral("host"))
                 .toString(),
             QStringLiteral("localhost"));
}

// A lista e' de NOMES, e um core que responda `Token` em vez de `token` nao
// pode abrir buraco.
void TestLogRedaction::field_names_are_matched_without_case()
{
    QVERIFY(isSecretField(QStringLiteral("Token")));
    QVERIFY(isSecretField(QStringLiteral("PASSWORD")));
    QVERIFY(isSecretField(QStringLiteral("Secret")));
    QVERIFY(isSecretField(QStringLiteral("credential")));
    QVERIFY(!isSecretField(QStringLiteral("tokenSource")));
    QVERIFY(!isSecretField(QStringLiteral("tokenVariable")));
}

// `tokenSource` e `tokenVariable` PRECISAM sobreviver: um e' a politica, o
// outro e' o NOME da variavel de ambiente. Nenhum dos dois e' o segredo, e
// redigi-los apagaria justamente o que explica de onde ele veio.
void TestLogRedaction::ordinary_fields_are_preserved()
{
    const QJsonObject limpo =
        redactSecrets(pedido(QStringLiteral("grafana.save"),
                             QJsonObject{
                                 {QStringLiteral("url"), QStringLiteral("http://localhost:3000")},
                                 {QStringLiteral("tokenSource"), QStringLiteral("environment")},
                                 {QStringLiteral("tokenVariable"), QStringLiteral("GRAFANA_TOKEN")},
                             }));
    const QJsonObject params = limpo.value(QStringLiteral("params")).toObject();
    QCOMPARE(params.value(QStringLiteral("tokenSource")).toString(),
             QStringLiteral("environment"));
    QCOMPARE(params.value(QStringLiteral("tokenVariable")).toString(),
             QStringLiteral("GRAFANA_TOKEN"));
}

// Profundidade qualquer, inclusive dentro de array: um segredo em lote e'
// segredo do mesmo jeito.
void TestLogRedaction::nested_objects_and_arrays_are_reached()
{
    const QString segredo = QStringLiteral("nao-deveria-sair");
    const QJsonObject limpo = redactSecrets(pedido(
        QStringLiteral("workspace.restore"),
        QJsonObject{
            {QStringLiteral("conexoes"),
             QJsonArray{
                 QJsonObject{{QStringLiteral("nome"), QStringLiteral("prod")},
                             {QStringLiteral("secret"), segredo}},
                 QJsonObject{{QStringLiteral("nome"), QStringLiteral("dev")},
                             {QStringLiteral("aninhado"),
                              QJsonObject{{QStringLiteral("token"), segredo}}}},
             }},
        }));
    QVERIFY(!comoTexto(limpo).contains(segredo));
}

// A politica `none` nao manda campo nenhum, e redigir nao pode INVENTAR um: um
// `"token":"***"` no log diria que houve credencial onde nao houve.
void TestLogRedaction::absent_token_stays_absent()
{
    const QJsonObject limpo = redactSecrets(pedido(QStringLiteral("grafana.probe"), QJsonObject{}));
    QVERIFY(!limpo.value(QStringLiteral("params")).toObject().contains(QStringLiteral("token")));
}

QTEST_GUILESS_MAIN(TestLogRedaction)

#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wctad-maybe-unsupported"
#endif
#include "tst_log_redaction.moc"
#ifdef __clang__
#pragma clang diagnostic pop
#endif
