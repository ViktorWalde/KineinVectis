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

using kinein::cli::Acao;
using kinein::cli::interpretar;
using kinein::cli::validarPasta;

class TestCliArgs : public QObject
{
    Q_OBJECT

private slots:
    void sem_argumento_nao_inventa_projeto();
    void caminho_relativo_resolve_contra_o_terminal_de_origem();
    void espacos_e_unicode_sobrevivem();
    void ajuda_e_versao_nao_sobem_nada();
    void opcao_desconhecida_e_recusada_em_vez_de_ignorada();
    void dois_caminhos_sao_recusados_com_os_dois_nomes();
    void o_separador_entrega_o_resto_ao_qt();
    void o_disco_diz_o_que_esta_errado();
};

void TestCliArgs::sem_argumento_nao_inventa_projeto()
{
    // A §3 da especificacao avisa: "abertura do desktop sem path nao deve
    // tratar um CWD arbitrario como projeto". O atalho do menu roda o binario
    // sem argumento, de um diretorio qualquer. Virar CWD e' decisao do COMANDO
    // CURTO (sugestao da §7), nao do binario.
    const auto r = interpretar({}, QStringLiteral("/home/u/proj"));
    QCOMPARE(r.acao, Acao::SemPasta);
    QVERIFY(r.pasta.isEmpty());
}

void TestCliArgs::caminho_relativo_resolve_contra_o_terminal_de_origem()
{
    const QString cwd = QStringLiteral("/home/u/proj");
    QCOMPARE(interpretar({QStringLiteral(".")}, cwd).pasta, cwd);
    QCOMPARE(interpretar({QStringLiteral("sub")}, cwd).pasta, QStringLiteral("/home/u/proj/sub"));
    QCOMPARE(interpretar({QStringLiteral("../outro")}, cwd).pasta, QStringLiteral("/home/u/outro"));
    // Absoluto nao e' reinterpretado, so' normalizado.
    QCOMPARE(interpretar({QStringLiteral("/tmp/./x/")}, cwd).pasta, QStringLiteral("/tmp/x"));
}

void TestCliArgs::espacos_e_unicode_sobrevivem()
{
    // A §3 da especificacao exige os dois, por escrito.
    const QString cwd = QStringLiteral("/home/u");
    QCOMPARE(interpretar({QStringLiteral("meu projeto")}, cwd).pasta,
             QStringLiteral("/home/u/meu projeto"));
    QCOMPARE(interpretar({QString::fromUtf8("projeto-ação")}, cwd).pasta,
             QString::fromUtf8("/home/u/projeto-ação"));
}

void TestCliArgs::ajuda_e_versao_nao_sobem_nada()
{
    for (const auto& forma : {QStringLiteral("--help"), QStringLiteral("-h")}) {
        const auto r = interpretar({forma}, QStringLiteral("/x"));
        QCOMPARE(r.acao, Acao::Ajuda);
        QVERIFY(r.mensagem.contains(QStringLiteral("uso:")));
        QVERIFY(r.pasta.isEmpty());
    }
    for (const auto& forma : {QStringLiteral("--version"), QStringLiteral("-V")}) {
        QCOMPARE(interpretar({forma}, QStringLiteral("/x")).acao, Acao::Versao);
    }
    // Ajuda vence um caminho na mesma linha: quem pede ajuda nao quer abrir.
    QCOMPARE(
        interpretar({QStringLiteral("/tmp"), QStringLiteral("--help")}, QStringLiteral("/x")).acao,
        Acao::Ajuda);
}

void TestCliArgs::opcao_desconhecida_e_recusada_em_vez_de_ignorada()
{
    const auto r = interpretar({QStringLiteral("--reuse-window")}, QStringLiteral("/x"));
    QCOMPARE(r.acao, Acao::Recusa);
    // A mensagem tem de NOMEAR o que nao entendeu, senao nao ajuda ninguem.
    QVERIFY(r.mensagem.contains(QStringLiteral("--reuse-window")));
    QVERIFY(r.mensagem.contains(QStringLiteral("--help")));
}

void TestCliArgs::dois_caminhos_sao_recusados_com_os_dois_nomes()
{
    // Varias raizes e' decisao ABERTA na §7; recusar dizendo por que e' honesto,
    // escolher uma calado nao e'.
    const auto r = interpretar({QStringLiteral("/a"), QStringLiteral("/b")}, QStringLiteral("/x"));
    QCOMPARE(r.acao, Acao::Recusa);
    QVERIFY(r.mensagem.contains(QStringLiteral("/a")));
    QVERIFY(r.mensagem.contains(QStringLiteral("/b")));
}

void TestCliArgs::o_separador_entrega_o_resto_ao_qt()
{
    // Depois de `--` vem argumento do Qt (`-platform offscreen`, por exemplo):
    // nada dali e' pasta, e nada dali pode virar recusa nossa.
    const auto r = interpretar({QStringLiteral("/tmp"), QStringLiteral("--"),
                                QStringLiteral("-platform"), QStringLiteral("offscreen")},
                               QStringLiteral("/x"));
    QCOMPARE(r.acao, Acao::Abrir);
    QCOMPARE(r.pasta, QStringLiteral("/tmp"));
}

void TestCliArgs::o_disco_diz_o_que_esta_errado()
{
    QTemporaryDir base;
    QVERIFY(base.isValid());
    const QDir raiz{base.path()};

    // Pasta vazia ABRE: a especificacao diz que nao se exige manifesto.
    QVERIFY(raiz.mkdir(QStringLiteral("vazia")));
    QVERIFY(validarPasta(raiz.filePath(QStringLiteral("vazia"))).isEmpty());

    // Inexistente: recusa NOMEANDO o caminho, e sem criar nada.
    const QString fantasma = raiz.filePath(QStringLiteral("nao-existe"));
    const QString erro = validarPasta(fantasma);
    QVERIFY(erro.contains(QStringLiteral("nao existe")));
    QVERIFY(erro.contains(fantasma));
    QVERIFY2(!QFileInfo::exists(fantasma), "validar NAO pode criar a pasta que faltava");

    // Arquivo no lugar de pasta: dizer qual e' o problema, nao so' falhar.
    const QString arquivo = raiz.filePath(QStringLiteral("um.txt"));
    QFile f{arquivo};
    QVERIFY(f.open(QIODevice::WriteOnly));
    f.close();
    QVERIFY(validarPasta(arquivo).contains(QStringLiteral("arquivo")));
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
