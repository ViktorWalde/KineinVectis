// Runner dos harnesses de logica QML (scripts/qml-harness/tst_*.qml).
//
// Por que este arquivo existe: ate o Debian 12 o gate usava o binario `qml`
// do pacote qt6-declarative-dev-tools. O Debian 13 parou de empacota-lo, e o
// gate de logica QML — 19 harnesses, a unica rede que EXECUTA o estado da UI —
// simplesmente nao rodava mais na maquina de desenvolvimento. Um gate que
// depende de uma ferramenta que a distro tira e poe nao e uma rede: e uma
// rede intermitente, que e pior, porque some justamente quando o ambiente
// muda e o risco de regressao e maior.
//
// O runner faz o minimo que o `qml` fazia para o nosso caso: sobe uma
// QGuiApplication, carrega UM arquivo .qml e devolve o codigo de saida que o
// `Qt.exit()` do harness pediu. Nao substitui o `qml` em nada alem disso.
//
// ATENCAO ao codigo de saida: POSIX guarda 8 bits, entao `Qt.exit(256)` chega
// como 0 — foi assim que 7 de 14 harnesses ficaram com checks que nunca
// reprovavam (docsprivate/AGENTS.md, ancora 3). Por isso os harnesses saem com
// 0 ou 1 e imprimem o bitmask por console.error; este runner nao mexe no
// codigo, so o repassa.

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QStringList>
#include <QUrl>
#include <QtGlobal>

int main(int argc, char* argv[])
{
    QGuiApplication app(argc, argv);

    // arguments() em vez de argv[1]: o clang-tidy do projeto proibe aritmetica
    // de ponteiro, e a lista do Qt ja resolve encoding da linha de comando.
    const QStringList argumentos = QGuiApplication::arguments();
    if (argumentos.size() < 2) {
        qWarning("uso: kinein-qml-harness <arquivo.qml>");
        return 2;
    }

    QQmlApplicationEngine engine;
    const QUrl arquivo = QUrl::fromLocalFile(argumentos.at(1));

    // Falha de carga (erro de sintaxe, import quebrado, tipo inexistente) tem
    // que REPROVAR. Sem isto um harness que nem carrega sairia com 0 e o gate
    // ficaria verde por nao ter olhado nada.
    bool falhouAoCarregar = false;
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app,
                     [&falhouAoCarregar]() {
                         falhouAoCarregar = true;
                         QGuiApplication::exit(3);
                     });

    engine.load(arquivo);
    if (falhouAoCarregar) {
        return 3;
    }

    return QGuiApplication::exec();
}
