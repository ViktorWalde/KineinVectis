// Primeiro teste de C++ do projeto (2026-07-17).
//
// POR QUE ELE EXISTE, e por que este alvo. Ate hoje a `ui/src` nao tinha teste
// NENHUM: os gates de C++ sao clang-format e clang-tidy, ambos estaticos, e os
// harnesses QML evitam o modulo C++ por construcao (nenhum dos 15 importa
// `KineinVectis`) — e' o que os deixa testaveis com fakes. Resultado: os tres
// maiores arquivos do repositorio nao tinham como reprovar.
//
// `languageForPath` foi escolhida por RESPONSABILIDADE, nao por conveniencia:
// "que linguagem e' este caminho?" e' uma pergunta inteira, com resposta pura
// (sem documento, sem event loop, sem QML), morando dentro de um arquivo de 818
// linhas de codigo que faz realce. E' a costura natural do primeiro corte do
// highlighter — o teste nasce antes da refatoracao de proposito, para que a
// refatoracao tenha rede.
//
// Alcancada pela API PUBLICA (`setFilePath` -> `language()`), nao pela estatica
// privada: teste que espia o interno congela a implementacao e depois impede o
// proprio split que ele deveria proteger.

#include "editor_highlighter.h"

#include <QObject>
#include <QString>
#include <QTest>

using kinein::EditorHighlighter;

class TestEditorHighlighter : public QObject
{
    Q_OBJECT

private slots:
    // A tabela e' o contrato: caminho -> linguagem. Cada linha aqui e' uma
    // afirmacao sobre o que a IDE realca, e falha sozinha se alguem mexer.
    void languageForPath_data()
    {
        QTest::addColumn<QString>("path");
        QTest::addColumn<QString>("expected");

        QTest::newRow("rust") << "/w/src/main.rs" << "rust";
        QTest::newRow("cpp") << "/w/ui/src/core_client.cpp" << "cpp";
        QTest::newRow("header .h") << "/w/ui/src/core_client.h" << "cpp";
        QTest::newRow("header .hpp") << "/w/x.hpp" << "cpp";
        QTest::newRow("c puro") << "/w/x.c" << "cpp";

        // CMakeLists.txt e' por NOME, nao por sufixo: o sufixo dele e' "txt", e
        // realcar CMake como texto puro seria o comportamento errado mais facil
        // de introduzir num refactor.
        QTest::newRow("CMakeLists por nome") << "/w/CMakeLists.txt" << "cmake";
        QTest::newRow("CMakeLists case-insensitive") << "/w/cmakelists.txt" << "cmake";

        // Extensao desconhecida cai em "plain" — NAO numa linguagem qualquer.
        // Realce errado mente com confianca, e "plain" e' a unica resposta
        // honesta para "nao sei o que e' isto".
        QTest::newRow("desconhecida vira plain") << "/w/LICENSE" << "plain";
        QTest::newRow("sem extensao vira plain") << "/w/Makefile" << "plain";
    }

    void languageForPath()
    {
        QFETCH(QString, path);
        QFETCH(QString, expected);

        EditorHighlighter highlighter;
        highlighter.setFilePath(path);

        QCOMPARE(highlighter.language(), expected);
    }

    // O highlighter vive sem documento durante a construcao da UI (o QML liga
    // `document` depois). Se `setFilePath` explodir aqui, a IDE nao abre —
    // e nenhum gate atual perceberia.
    void sobreviveSemDocumento()
    {
        EditorHighlighter highlighter;
        QVERIFY(highlighter.quickDocument() == nullptr);
        highlighter.setFilePath("/w/src/main.rs");
        QCOMPARE(highlighter.language(), QStringLiteral("rust"));
    }

    // `setFilePath` sai cedo quando o caminho nao mudou. Isso e' o que faz um
    // caminho VAZIO nao virar "plain": o objeto nasce com m_filePath vazio, a
    // guarda dispara e a linguagem fica no valor inicial ("").
    //
    // Este caso esta aqui porque a primeira versao deste teste o afirmava dentro
    // da tabela de languageForPath e passava — pelo motivo errado. Nao exercitava
    // languageForPath nenhuma; media a guarda achando que media a tabela. E' a
    // §0.2i do PONTO_ATUAL renascendo em C++ no primeiro teste da linguagem, e
    // fica registrado para o proximo nao repetir: teste que passa por acidente e'
    // pior que teste ausente, porque compra confianca sem entregar nada.
    void caminhoIgualNaoRecalcula()
    {
        EditorHighlighter highlighter;
        QCOMPARE(highlighter.language(), QString()); // valor inicial, nao "plain"

        highlighter.setFilePath(QString()); // guarda: nao mudou
        QCOMPARE(highlighter.language(), QString());

        highlighter.setFilePath("/w/x.rs");
        QCOMPARE(highlighter.language(), QStringLiteral("rust"));
        highlighter.setFilePath("/w/x.rs"); // de novo: sem efeito
        QCOMPARE(highlighter.language(), QStringLiteral("rust"));
    }
};

QTEST_MAIN(TestEditorHighlighter)
#include "tst_editor_highlighter.moc"
