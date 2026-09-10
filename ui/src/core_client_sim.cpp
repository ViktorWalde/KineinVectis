// Dominio de SIMULACAO no lado da UI: pedidos e respostas de `sim.*`.
//
// Arquivo proprio pela mesma regra do `core_client_grafana.cpp`: a §5 da
// ARCHITECTURE manda dividir o `CoreClient` por dominio, e dominio novo ganha
// arquivo novo em vez de engordar o vizinho.
//
// Nada aqui decide nada. Quem sabe o que e' um conceito, o que a formula usa e
// se ela bate com o conceito e' o core (`crates/kinein-core/src/sim/`). Este
// arquivo leva o pedido e devolve o que voltou — inclusive a RECUSA, porque
// neste dominio a recusa e' informacao de produto: "falta o valor de v" e "o
// resultado nao e' um numero utilizavel" sao coisas que o autor precisa ler.
// NOTA: o parametro chama-se `conceptId`, e nao `concept`, porque `concept` e'
// PALAVRA RESERVADA do C++20. A chave JSON continua sendo `concept`, que e' o
// nome do contrato — o que muda e' so' o identificador C++.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

// A CHECAGEM RODA A CADA TECLA. O core mede 5,3 us por checagem, entao o custo
// esta' na ida e volta pelo JSON-RPC, nao na conta — por isso o controller
// espera 180 ms antes de perguntar, o mesmo respiro do observador de arquivos.
//
// `conceptId` e nao `concept` nas assinaturas abaixo: `concept` e' palavra
// RESERVADA do C++20. A chave JSON continua sendo `concept`, que e' o nome do
// contrato; o que muda e' so' o identificador C++.
//
// A ESTIMATIVA e' pedida ANTES de rodar: a IDE mostra o custo e o usuario
// escolhe (arquitetura/34 §2.1). Contar passos e' regra de negocio, e por isso
// a conta mora no core e nao no QML.
//
// A PERSISTENCIA guarda o que o autor MONTOU, e nunca o que a maquina produziu
// (arquitetura/34 §9) — a licao do `.ipynb`.
void CoreClient::simCatalog(const QString& course)
{
    QJsonObject params;
    // Campo ausente e campo vazio sao coisas diferentes: ausente significa
    // "quero todos", e e' o que a tela pede ao abrir.
    if (!course.isEmpty()) {
        params.insert(QStringLiteral("course"), course);
    }
    sendRequest(QStringLiteral("sim.catalog"), params);
}

void CoreClient::simInspectFormula(const QString& formula)
{
    sendRequest(QStringLiteral("sim.inspectFormula"),
                QJsonObject{{QStringLiteral("formula"), formula}});
}

void CoreClient::simCheckFormula(const QString& conceptId, const QString& formula,
                                 const QVariantList& bindings)
{
    sendRequest(QStringLiteral("sim.checkFormula"),
                QJsonObject{{QStringLiteral("concept"), conceptId},
                            {QStringLiteral("formula"), formula},
                            {QStringLiteral("bindings"), QJsonArray::fromVariantList(bindings)}});
}

void CoreClient::simEvaluate(const QString& conceptId, const QString& formula,
                             const QVariantList& bindings, const QVariantList& values)
{
    sendRequest(QStringLiteral("sim.evaluate"),
                QJsonObject{{QStringLiteral("concept"), conceptId},
                            {QStringLiteral("formula"), formula},
                            {QStringLiteral("bindings"), QJsonArray::fromVariantList(bindings)},
                            {QStringLiteral("values"), QJsonArray::fromVariantList(values)}});
}

void CoreClient::simEstimate(double duration, double step, int samples)
{
    sendRequest(QStringLiteral("sim.estimate"), QJsonObject{{QStringLiteral("duration"), duration},
                                                            {QStringLiteral("step"), step},
                                                            {QStringLiteral("samples"), samples}});
}

void CoreClient::simRun(const QString& conceptId, const QString& formula,
                        const QVariantList& bindings, const QVariantList& values,
                        const QVariantMap& initial, double duration, double step,
                        const QString& method, int samples)
{
    sendRequest(QStringLiteral("sim.run"),
                QJsonObject{{QStringLiteral("concept"), conceptId},
                            {QStringLiteral("formula"), formula},
                            {QStringLiteral("bindings"), QJsonArray::fromVariantList(bindings)},
                            {QStringLiteral("values"), QJsonArray::fromVariantList(values)},
                            {QStringLiteral("initial"), QJsonObject::fromVariantMap(initial)},
                            {QStringLiteral("duration"), duration},
                            {QStringLiteral("step"), step},
                            {QStringLiteral("method"), method},
                            {QStringLiteral("samples"), samples}});
}

// A FORMA VETORIAL. `equations` chega como lista de mapas
// { component, formula, bindings } — e a ordem dela NAO importa: o core casa
// pelo campo `component`, porque supor que a n-esima formula e' do n-esimo
// componente seria adivinhar (arquitetura/34 §13.2).
void CoreClient::simCheckSystem(const QString& conceptId, const QVariantList& equations)
{
    sendRequest(QStringLiteral("sim.checkSystem"),
                QJsonObject{{QStringLiteral("concept"), conceptId},
                            {QStringLiteral("equations"), QJsonArray::fromVariantList(equations)}});
}

void CoreClient::simRunSystem(const QString& conceptId, const QVariantList& equations,
                              const QVariantList& values, const QVariantList& initial,
                              double duration, double step, const QString& method, int samples)
{
    sendRequest(QStringLiteral("sim.runSystem"),
                QJsonObject{{QStringLiteral("concept"), conceptId},
                            {QStringLiteral("equations"), QJsonArray::fromVariantList(equations)},
                            {QStringLiteral("values"), QJsonArray::fromVariantList(values)},
                            {QStringLiteral("initial"), QJsonArray::fromVariantList(initial)},
                            {QStringLiteral("duration"), duration},
                            {QStringLiteral("step"), step},
                            {QStringLiteral("method"), method},
                            {QStringLiteral("samples"), samples}});
}

void CoreClient::simList()
{
    sendRequest(QStringLiteral("sim.list"), QJsonObject{});
}

void CoreClient::simSave(const QVariantMap& simulation)
{
    sendRequest(QStringLiteral("sim.save"), QJsonObject{{QStringLiteral("simulation"),
                                                         QJsonObject::fromVariantMap(simulation)}});
}

void CoreClient::simForget(const QString& name)
{
    sendRequest(QStringLiteral("sim.forget"), QJsonObject{{QStringLiteral("name"), name}});
}

bool CoreClient::dispatchSimResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("sim.catalog")) {
        emit simCatalogResolved(result.value(QStringLiteral("concepts")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("sim.inspectFormula")) {
        emit simFormulaInspected(
            result.value(QStringLiteral("variables")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("sim.checkFormula")) {
        // O resultado inteiro atravessa, e nao so' o booleano: cada problema
        // tem `kind` proprio para a tela nao ter de casar por TEXTO. Foi esse
        // casamento que o `requestFailed` sem `code` obrigou em 2026-09-04.
        emit simFormulaChecked(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("sim.evaluate")) {
        emit simEvaluated(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("sim.estimate")) {
        emit simEstimated(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("sim.checkSystem")) {
        emit simSystemChecked(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("sim.runSystem")) {
        emit simSystemRan(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("sim.run")) {
        emit simRan(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("sim.list") || method == QStringLiteral("sim.save") ||
        method == QStringLiteral("sim.forget"))
    {
        // Os tres devolvem a LISTA inteira: salvar e esquecer nao pedem que a
        // tela adivinhe o novo estado a partir do antigo.
        emit simSavedListResolved(
            result.value(QStringLiteral("simulations")).toArray().toVariantList());
        return true;
    }
    return false;
}

} // namespace kinein
