// UMA JANELA POR PASTA — a parte que decide, medida sem abrir janela nenhuma.
//
// Por que existe (2026-09-26): o autor escolheu "mesma pasta foca a janela
// aberta; outra pasta abre nova". O que torna isso seguro nao e' o foco — e'
// NAO abrir a segunda janela, porque nao ha' lock de workspace e duas janelas
// na mesma pasta escrevem o mesmo `.kinein/session.json`, com a ultima a
// fechar apagando o que a outra gravou.
//
// Aqui ficam as decisoes puras: o nome do socket, a mensagem, a conferencia do
// caminho e a leitura da resposta. O socket de verdade e a ativacao da janela
// vivem no `main`, onde ha' sistema operacional e compositor.
#include "single_instance.h"

#include <QEventLoop>
#include <QFileInfo>
#include <QTemporaryDir>
#include <QTimer>
#include <QtTest>

#include <thread>

#ifdef Q_OS_UNIX
#include <unistd.h>
#endif

using kinein::answerIsMine;
using kinein::encodeRequest;
using kinein::mineAnswer;
using kinein::notMineAnswer;
using kinein::parseRequest;
using kinein::sameWorkspace;
using kinein::socketPathFor;

namespace {

// O DONO ESPERA PELO MESMO CAMINHO DA PRODUCAO: a escuta do `watchIncoming`
// (o QSocketNotifier no Unix, o sinal do QLocalServer no Windows) e so' entao
// o `acceptOne`. Chama-lo antes da conexao chegar mediria o scheduler, nao o
// protocolo. Quem chega roda numa thread, com o laco de eventos do dono aqui.
std::optional<kinein::Request> acceptWhenReady(int fd, const QString& workspace)
{
    QEventLoop loop;
    bool ready = false;
    const std::unique_ptr<QObject> watcher = kinein::watchIncoming(fd, [&] {
        ready = true;
        loop.quit();
    });
    if (!watcher) {
        return std::nullopt;
    }
    QTimer::singleShot(3000, &loop, &QEventLoop::quit);
    loop.exec();
    return ready ? kinein::acceptOne(fd, workspace) : std::nullopt;
}

} // namespace

class TestSingleInstance : public QObject
{
    Q_OBJECT

private slots:
    void the_socket_name_is_short_no_matter_how_deep_the_project_is();
    void different_folders_never_share_a_socket_name();
    void without_a_runtime_dir_there_is_no_socket();
    void a_request_survives_spaces_and_unicode_in_the_path();
    void a_line_from_another_protocol_is_refused();
    void a_request_cannot_smuggle_a_second_line();
    void the_same_folder_written_two_ways_is_the_same_folder();
    void only_the_exact_yes_counts_as_yes();
    void a_live_owner_answers_and_the_newcomer_steps_aside();
    void an_owner_of_another_folder_says_no_right_away();
    void an_orphan_socket_is_taken_over();
    void with_nobody_listening_the_newcomer_just_opens();
};

// `sun_path` tem 108 bytes no Linux. Um projeto fundo nao pode estourar isso,
// e e' por isso que o nome do arquivo e' um digest e nao o caminho escapado.
void TestSingleInstance::the_socket_name_is_short_no_matter_how_deep_the_project_is()
{
    QString deepPath = QStringLiteral("/home/alguem");
    for (int level = 0; level < 40; ++level) {
        deepPath += QStringLiteral("/uma-pasta-com-nome-bem-comprido-%1").arg(level);
    }
    const QString socket = socketPathFor(QStringLiteral("/run/user/1000"), deepPath);
    QVERIFY(!socket.isEmpty());
    QVERIFY2(
        socket.toUtf8().size() < 100,
        qPrintable(
            QStringLiteral("socket com %1 bytes: %2").arg(socket.toUtf8().size()).arg(socket)));
}

void TestSingleInstance::different_folders_never_share_a_socket_name()
{
    const QString first =
        socketPathFor(QStringLiteral("/run/user/1000"), QStringLiteral("/home/alguem/projeto-a"));
    const QString second =
        socketPathFor(QStringLiteral("/run/user/1000"), QStringLiteral("/home/alguem/projeto-b"));
    QVERIFY(!first.isEmpty());
    QCOMPARE_NE(first, second);
    // E a MESMA pasta da' sempre o mesmo nome — senao a coordenacao nao existe.
    QCOMPARE(first, socketPathFor(QStringLiteral("/run/user/1000"),
                                  QStringLiteral("/home/alguem/projeto-a")));
}

// Sessao sem `XDG_RUNTIME_DIR` existe (container magro, sessao sem systemd).
// Seguir sem coordenacao e' melhor que inventar um diretorio onde escrever.
void TestSingleInstance::without_a_runtime_dir_there_is_no_socket()
{
    QVERIFY(socketPathFor(QString{}, QStringLiteral("/home/alguem/p")).isEmpty());
    QVERIFY(socketPathFor(QStringLiteral("   "), QStringLiteral("/home/alguem/p")).isEmpty());
    QVERIFY(socketPathFor(QStringLiteral("/run/user/1000"), QString{}).isEmpty());
}

void TestSingleInstance::a_request_survives_spaces_and_unicode_in_the_path()
{
    const QString folderPath = QStringLiteral("/home/alguém/meus projetos/ção");
    const auto parsed = parseRequest(encodeRequest(folderPath, QStringLiteral("tok-123")));
    QVERIFY(parsed.has_value());
    QCOMPARE(parsed->workspacePath, folderPath);
    QCOMPARE(parsed->activationToken, QStringLiteral("tok-123"));

    // Sem token tambem e' um pedido legitimo: nem todo terminal fornece um.
    const auto withoutToken = parseRequest(encodeRequest(folderPath, QString{}));
    QVERIFY(withoutToken.has_value());
    QVERIFY(withoutToken->activationToken.isEmpty());
}

// Um socket no `XDG_RUNTIME_DIR` e' alcancavel por qualquer processo do mesmo
// usuario. Responder a qualquer coisa que chegue seria obedecer a quem nao se
// identificou.
void TestSingleInstance::a_line_from_another_protocol_is_refused()
{
    QVERIFY(!parseRequest(QStringLiteral("GET / HTTP/1.1\n")).has_value());
    QVERIFY(!parseRequest(QString{}).has_value());
    QVERIFY(!parseRequest(QStringLiteral("KINEIN-INSTANCIA-1\n")).has_value());
    QVERIFY(!parseRequest(QStringLiteral("KINEIN-INSTANCIA-9\t/p\t\n")).has_value());
    // Prefixo certo e caminho vazio nao e' pedido: nao ha' pasta para conferir.
    QVERIFY(!parseRequest(QStringLiteral("KINEIN-INSTANCIA-1\t   \t\n")).has_value());
}

// Quebra de linha dentro do campo viraria uma SEGUNDA mensagem do outro lado.
// O codificador as remove, e este teste guarda isso.
void TestSingleInstance::a_request_cannot_smuggle_a_second_line()
{
    const QString poison = QStringLiteral("/p\nKINEIN-INSTANCIA-1\t/outra\t");
    const QString encoded = encodeRequest(poison, QStringLiteral("t\nok"));
    QCOMPARE(encoded.count(QLatin1Char('\n')), 1);
    const auto parsed = parseRequest(encoded);
    QVERIFY(!parsed.has_value() || parsed->workspacePath != QStringLiteral("/outra"));
}

void TestSingleInstance::the_same_folder_written_two_ways_is_the_same_folder()
{
    QVERIFY(sameWorkspace(QStringLiteral("/home/a/p"), QStringLiteral("/home/a/p/")));
    QVERIFY(sameWorkspace(QStringLiteral("/home/a/p"), QStringLiteral("/home/a/./p")));
    QVERIFY(sameWorkspace(QStringLiteral("  /home/a/p  "), QStringLiteral("/home/a/p")));
    QVERIFY(!sameWorkspace(QStringLiteral("/home/a/p"), QStringLiteral("/home/a/p2")));
    // Vazio nao e' igual a vazio: "nao sei qual pasta" nao pode virar "a mesma".
    QVERIFY(!sameWorkspace(QString{}, QString{}));
}

void TestSingleInstance::only_the_exact_yes_counts_as_yes()
{
    QVERIFY(answerIsMine(mineAnswer()));
    QVERIFY(!answerIsMine(notMineAnswer()));
    QVERIFY(!answerIsMine(QString{}));
    QVERIFY(!answerIsMine(QStringLiteral("MEUS\n")));
    QVERIFY(!answerIsMine(QStringLiteral("talvez\n")));
}

// DAQUI PARA BAIXO HA' SOCKET DE VERDADE. Nada de mock: o que se mede e' o
// aperto de mao completo, com dois lados e prazo.

// O caso que a decisao do autor pede: `kinein ~/projeto` com o projeto ja'
// aberto NAO abre a segunda janela.
void TestSingleInstance::a_live_owner_answers_and_the_newcomer_steps_aside()
{
    QTemporaryDir runtime;
    QVERIFY(runtime.isValid());
    const QString project = QStringLiteral("/home/alguem/projeto");
    const QString socket = socketPathFor(runtime.path(), project);

    const int ownerFd = kinein::listenFor(socket);
    QVERIFY2(ownerFd >= 0, "nao consegui escutar no socket");

    bool handedOff = false;
    std::thread newcomer(
        [&] { handedOff = kinein::handOff(socket, project, QStringLiteral("token-do-terminal")); });
    const std::optional<kinein::Request> received = acceptWhenReady(ownerFd, project);
    newcomer.join();

    QVERIFY2(handedOff, "o recem-chegado devia sair, e nao abrir outra janela");
    QVERIFY(received.has_value());
    QCOMPARE(received->activationToken, QStringLiteral("token-do-terminal"));
    kinein::releaseSocket(ownerFd, socket);
}

// COLISAO DE NOME NAO PODE FAZER UMA PASTA SE PASSAR POR OUTRA. O caminho
// inteiro viaja na mensagem e e' conferido, entao o dono de outra pasta diz
// nao — e diz RAPIDO, em vez de deixar o outro lado esperar o prazo.
void TestSingleInstance::an_owner_of_another_folder_says_no_right_away()
{
    QTemporaryDir runtime;
    QVERIFY(runtime.isValid());
    const QString socket = socketPathFor(runtime.path(), QStringLiteral("/qualquer"));
    const int ownerFd = kinein::listenFor(socket);
    QVERIFY(ownerFd >= 0);

    // O QUE SE MEDE e' "respondeu ANTES do prazo", e nao um numero de
    // milissegundos. Por isso o prazo aqui e' folgado de proposito: com o
    // limite colado no tempo real, o teste piscava quando a maquina estava
    // ocupada — visto em 2026-09-26, com o clang-tidy rodando ao lado. Teste
    // que falha ao acaso ensina a ignorar teste.
    constexpr int kGenerousTimeoutMs = 3000;
    bool handedOff = true;
    qint64 elapsedMs = 0;
    std::thread newcomer([&] {
        QElapsedTimer clock;
        clock.start();
        handedOff = kinein::handOff(socket, QStringLiteral("/home/alguem/projeto"), QString{},
                                    kGenerousTimeoutMs);
        elapsedMs = clock.elapsed();
    });
    const std::optional<kinein::Request> received =
        acceptWhenReady(ownerFd, QStringLiteral("/home/alguem/OUTRA"));
    newcomer.join();

    QVERIFY2(!handedOff, "pasta diferente tem de abrir janela nova");
    QVERIFY(!received.has_value());
    QVERIFY2(elapsedMs < kGenerousTimeoutMs / 2,
             qPrintable(QStringLiteral("a recusa esperou o prazo (%1 ms) em vez de vir na hora")
                            .arg(elapsedMs)));
    kinein::releaseSocket(ownerFd, socket);
}

// Depois de um crash sobra o arquivo do socket sem ninguem atras dele. Deixar
// a coordenacao quebrada ate' o proximo reboot seria pior que assumir o lugar.
//
// NO WINDOWS NAO HA' ORFAO: o pipe some com o ultimo handle, crash inclusive.
// O que se prova la' e' o equivalente — o dono que saiu libera o nome na hora,
// e o seguinte assume sem esperar nada.
void TestSingleInstance::an_orphan_socket_is_taken_over()
{
    QTemporaryDir runtime;
    QVERIFY(runtime.isValid());
    const QString project = QStringLiteral("/home/alguem/projeto");
    const QString socket = socketPathFor(runtime.path(), project);

    const int deadFd = kinein::listenFor(socket);
    QVERIFY(deadFd >= 0);
#ifdef Q_OS_UNIX
    // O processo morre sem limpar: o descritor fecha, o ARQUIVO fica.
    ::close(deadFd);
    QVERIFY(QFileInfo::exists(socket));
#else
    // Enquanto o dono vive, o nome e' dele: ninguem assume.
    QCOMPARE(kinein::listenFor(socket), -1);
    kinein::releaseSocket(deadFd, socket);
#endif

    const int freshFd = kinein::listenFor(socket);
    QVERIFY2(freshFd >= 0, "o lugar do dono que saiu nao foi assumido");
    kinein::releaseSocket(freshFd, socket);
    QVERIFY(!QFileInfo::exists(socket));
}

// Sem ninguem do outro lado, abrir e' a resposta certa — e sem esperar prazo
// nenhum, porque o terminal esta' na frente de alguem.
void TestSingleInstance::with_nobody_listening_the_newcomer_just_opens()
{
    QTemporaryDir runtime;
    QVERIFY(runtime.isValid());
    const QString socket = socketPathFor(runtime.path(), QStringLiteral("/home/a/p"));
    // Mesmo raciocinio: o ponto e' nao esperar o prazo, e nao um numero.
    constexpr int kGenerousTimeoutMs = 3000;
    QElapsedTimer clock;
    clock.start();
    QVERIFY(!kinein::handOff(socket, QStringLiteral("/home/a/p"), QString{}, kGenerousTimeoutMs));
    QVERIFY2(clock.elapsed() < kGenerousTimeoutMs / 2, "esperou o prazo com ninguem do outro lado");
}

QTEST_GUILESS_MAIN(TestSingleInstance)

#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wctad-maybe-unsupported"
#endif
#include "tst_single_instance.moc"
#ifdef __clang__
#pragma clang diagnostic pop
#endif
