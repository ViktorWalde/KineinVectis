// O PRIMEIRO teste C++ do projeto (2026-09-24).
//
// Ate' aqui o repo media Rust (`cargo test`) e QML (harnesses headless), e o C++
// da ponte era coberto so' por clang-tidy, pela catraca de fiacao IPC e pelo
// smoke de "o binario abre". Nenhuma dessas tres olha o que uma funcao decide.
// Nao havia decisao registrada justificando a ausencia — era omissao.
//
// O primeiro assunto coberto e' o contrato da linha de comando, porque ele e'
// puro (sem GUI, sem QML) e porque o comportamento antigo — varrer argv e
// ignorar em silencio o que nao fosse pasta existente — e' exatamente o tipo de
// defeito que nenhum dos tres gates anteriores pegaria.
#include "cli_args.h"

#include <QDir>
#include <QTemporaryDir>
#include <QtTest>

using kinein::cli::Action;
using kinein::cli::Arguments;
using kinein::cli::parse;
using kinein::cli::shouldDetach;
using kinein::cli::validateFolder;

class TestCliArgs : public QObject
{
    Q_OBJECT

private slots:
    void no_argument_invents_no_project();
    void relative_path_resolves_against_the_calling_terminal();
    void spaces_and_unicode_survive();
    void help_and_version_start_nothing();
    void unknown_option_is_refused_not_ignored();
    void two_paths_are_refused_naming_both();
    void the_separator_hands_the_rest_to_qt();
    void the_disk_says_what_is_wrong();
    void wait_and_verbose_are_parsed();
    void detaches_only_when_a_terminal_calls();
};

void TestCliArgs::no_argument_invents_no_project()
{
    // A §3 da especificacao avisa: "abertura do desktop sem path nao deve
    // tratar um CWD arbitrario como projeto". O atalho do menu roda o binario
    // sem argumento, de um diretorio qualquer. Virar CWD e' decisao do COMANDO
    // CURTO (sugestao da §7), nao do binario.
    const auto r = parse({}, QStringLiteral("/home/u/proj"));
    QCOMPARE(r.action, Action::NoFolder);
    QVERIFY(r.folder.isEmpty());
}

void TestCliArgs::relative_path_resolves_against_the_calling_terminal()
{
    const QString cwd = QStringLiteral("/home/u/proj");
    QCOMPARE(parse({QStringLiteral(".")}, cwd).folder, cwd);
    QCOMPARE(parse({QStringLiteral("sub")}, cwd).folder, QStringLiteral("/home/u/proj/sub"));
    QCOMPARE(parse({QStringLiteral("../outro")}, cwd).folder, QStringLiteral("/home/u/outro"));
    // Absoluto nao e' reinterpretado, so' normalizado.
    QCOMPARE(parse({QStringLiteral("/tmp/./x/")}, cwd).folder, QStringLiteral("/tmp/x"));
}

void TestCliArgs::spaces_and_unicode_survive()
{
    // A §3 da especificacao exige os dois, por escrito.
    const QString cwd = QStringLiteral("/home/u");
    QCOMPARE(parse({QStringLiteral("meu projeto")}, cwd).folder,
             QStringLiteral("/home/u/meu projeto"));
    QCOMPARE(parse({QString::fromUtf8("projeto-ação")}, cwd).folder,
             QString::fromUtf8("/home/u/projeto-ação"));
}

void TestCliArgs::help_and_version_start_nothing()
{
    for (const auto& spelling : {QStringLiteral("--help"), QStringLiteral("-h")}) {
        const auto r = parse({spelling}, QStringLiteral("/x"));
        QCOMPARE(r.action, Action::Help);
        QVERIFY(r.message.contains(QStringLiteral("uso:")));
        QVERIFY(r.folder.isEmpty());
    }
    for (const auto& spelling : {QStringLiteral("--version"), QStringLiteral("-V")}) {
        QCOMPARE(parse({spelling}, QStringLiteral("/x")).action, Action::Version);
    }
    // Ajuda vence um caminho na mesma linha: quem pede ajuda nao quer abrir.
    QCOMPARE(parse({QStringLiteral("/tmp"), QStringLiteral("--help")}, QStringLiteral("/x")).action,
             Action::Help);
}

void TestCliArgs::unknown_option_is_refused_not_ignored()
{
    const auto r = parse({QStringLiteral("--reuse-window")}, QStringLiteral("/x"));
    QCOMPARE(r.action, Action::Refusal);
    // A mensagem tem de NOMEAR o que nao entendeu, senao nao ajuda ninguem.
    QVERIFY(r.message.contains(QStringLiteral("--reuse-window")));
    QVERIFY(r.message.contains(QStringLiteral("--help")));
}

void TestCliArgs::two_paths_are_refused_naming_both()
{
    // Varias raizes e' decisao ABERTA na §7; recusar dizendo por que e' honesto,
    // escolher uma calado nao e'.
    const auto r = parse({QStringLiteral("/a"), QStringLiteral("/b")}, QStringLiteral("/x"));
    QCOMPARE(r.action, Action::Refusal);
    QVERIFY(r.message.contains(QStringLiteral("/a")));
    QVERIFY(r.message.contains(QStringLiteral("/b")));
}

void TestCliArgs::the_separator_hands_the_rest_to_qt()
{
    // Depois de `--` vem argumento do Qt (`-platform offscreen`, por exemplo):
    // nada dali e' pasta, e nada dali pode virar recusa nossa.
    const auto r = parse({QStringLiteral("/tmp"), QStringLiteral("--"), QStringLiteral("-platform"),
                          QStringLiteral("offscreen")},
                         QStringLiteral("/x"));
    QCOMPARE(r.action, Action::Open);
    QCOMPARE(r.folder, QStringLiteral("/tmp"));
}

void TestCliArgs::the_disk_says_what_is_wrong()
{
    QTemporaryDir base;
    QVERIFY(base.isValid());
    const QDir root{base.path()};

    // Pasta vazia ABRE: a especificacao diz que nao se exige manifesto.
    QVERIFY(root.mkdir(QStringLiteral("vazia")));
    QVERIFY(validateFolder(root.filePath(QStringLiteral("vazia"))).isEmpty());

    // Inexistente: recusa NOMEANDO o caminho, e sem criar nada.
    const QString missing = root.filePath(QStringLiteral("nao-existe"));
    const QString error = validateFolder(missing);
    QVERIFY(error.contains(QStringLiteral("nao existe")));
    QVERIFY(error.contains(missing));
    QVERIFY2(!QFileInfo::exists(missing), "validar NAO pode criar a pasta que faltava");

    // Arquivo no lugar de pasta: dizer qual e' o problema, nao so' falhar.
    const QString file = root.filePath(QStringLiteral("um.txt"));
    QFile f{file};
    QVERIFY(f.open(QIODevice::WriteOnly));
    f.close();
    QVERIFY(validateFolder(file).contains(QStringLiteral("arquivo")));
}

void TestCliArgs::wait_and_verbose_are_parsed()
{
    const QString cwd = QStringLiteral("/home/u/proj");
    const Arguments w = parse({QStringLiteral("-w"), QStringLiteral(".")}, cwd);
    QCOMPARE(w.action, Action::Open);
    QVERIFY(w.waitForClose);
    QVERIFY(!w.verbose);
    QCOMPARE(w.folder, cwd);
    const Arguments v = parse({QStringLiteral("--verbose")}, cwd);
    QCOMPARE(v.action, Action::NoFolder);
    QVERIFY(v.verbose);
    QVERIFY(parse({QStringLiteral("--wait"), QStringLiteral("x")}, cwd).waitForClose);
}

void TestCliArgs::detaches_only_when_a_terminal_calls()
{
    const QString cwd = QStringLiteral("/home/u/proj");
    const Arguments openRequest = parse({QStringLiteral(".")}, cwd);
    QVERIFY(shouldDetach(openRequest, true, false));
    // Menu, script e gate (stderr nao e' terminal) continuam como sempre.
    QVERIFY(!shouldDetach(openRequest, false, false));
    // O filho nao desacopla de novo.
    QVERIFY(!shouldDetach(openRequest, true, true));
    QVERIFY(!shouldDetach(parse({QStringLiteral("-w"), QStringLiteral(".")}, cwd), true, false));
    QVERIFY(!shouldDetach(parse({QStringLiteral("--verbose")}, cwd), true, false));
    // Ajuda, versao e recusa respondem no proprio terminal.
    QVERIFY(!shouldDetach(parse({QStringLiteral("--help")}, cwd), true, false));
    QVERIFY(!shouldDetach(parse({QStringLiteral("--version")}, cwd), true, false));
    QVERIFY(!shouldDetach(parse({QStringLiteral("--nao-existe")}, cwd), true, false));
}

QTEST_GUILESS_MAIN(TestCliArgs)

// Mesmo padrao que o `typing_perf_harness.cpp` ja' usa e explica: o `.moc` e'
// codigo GERADO, e o moc do Qt 6.10 monta os QtMocHelpers por CTAD, que o
// `-Wctad-maybe-unsupported` do Clang 21 reprova sob `-Werror`. A isencao fica
// presa ao include gerado — o resto do arquivo segue sob os avisos rigorosos.
#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wctad-maybe-unsupported"
#endif
#include "tst_cli_args.moc"
#ifdef __clang__
#pragma clang diagnostic pop
#endif
