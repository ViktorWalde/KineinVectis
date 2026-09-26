// O TERCEIRO teste C++ do projeto (2026-09-25).
//
// O assunto e' seguranca, e por isso entra com teste desde a primeira linha: a
// §5 da especificacao do preview de Markdown diz que ele NAO e' navegador, e
// que nao ha' "leitura fora do workspace por URL construida no documento". Um
// `.md` e' conteudo de terceiros; pode chegar num clone, num anexo, numa
// dependencia. As regras abaixo sao o que impede um documento de alcancar o
// que quiser da maquina de quem o abre.
#include "markdown_policy.h"

#include <QtTest>

using kinein::markdown::decideLink;
using kinein::markdown::decideResource;
using kinein::markdown::insideRoot;
using kinein::markdown::LinkKind;
using kinein::markdown::rewriteRefusedImages;

namespace {
const QString kRoot = QStringLiteral("/casa/projeto");
const QString kDoc = QStringLiteral("/casa/projeto/docs/guia.md");
} // namespace

class TestMarkdownPolicy : public QObject
{
    Q_OBJECT

private slots:
    void anchor_navigates_inside_the_preview();
    void relative_resolves_from_the_document_folder();
    void dot_dot_does_not_escape_silently();
    void a_similar_prefix_is_not_the_same_project();
    void web_opens_outside_and_only_http_and_https();
    void an_unknown_scheme_is_refused_by_name();
    void with_no_project_open_nothing_local_opens();
    void a_remote_image_is_blocked_by_default();
    void a_local_image_outside_the_project_is_not_read();
    void an_anchor_in_a_file_link_is_not_part_of_the_path();
    void a_refused_image_never_reaches_the_renderer();
    void a_code_fence_is_content_and_is_not_rewritten();
    void several_images_on_one_line_are_each_decided();
};

void TestMarkdownPolicy::anchor_navigates_inside_the_preview()
{
    const auto decision = decideLink(QStringLiteral("#secao-2"), kDoc, kRoot);
    QCOMPARE(decision.kind, LinkKind::Anchor);
    QCOMPARE(decision.target, QStringLiteral("secao-2"));
}

void TestMarkdownPolicy::relative_resolves_from_the_document_folder()
{
    const auto decision = decideLink(QStringLiteral("outro.md"), kDoc, kRoot);
    QCOMPARE(decision.kind, LinkKind::LocalFile);
    QCOMPARE(decision.target, QStringLiteral("/casa/projeto/docs/outro.md"));

    const auto upwards = decideLink(QStringLiteral("../README.md"), kDoc, kRoot);
    QCOMPARE(upwards.kind, LinkKind::LocalFile);
    QCOMPARE(upwards.target, QStringLiteral("/casa/projeto/README.md"));
}

void TestMarkdownPolicy::dot_dot_does_not_escape_silently()
{
    // O caso que a §4.1 nomeia: "`../` nao pode escapar silenciosamente do
    // workspace". Um `.md` de terceiros lendo `~/.ssh/id_rsa` comeca assim.
    //
    // O DOCUMENTO esta' em `/casa/projeto/docs`, entao a conta dos `..` parte
    // dali, e nao da raiz do projeto. Este teste nasceu errado por esquecer
    // isso: `docs/../../fora.md` resolve para `/casa/projeto/fora.md`, que esta'
    // DENTRO — e o codigo acertava ao permitir.
    for (const QString& href :
         {QStringLiteral("../../etc/passwd"), QStringLiteral("../../../casa/segredo.txt"),
          QStringLiteral("/etc/passwd"), QStringLiteral("docs/../../../fora.md")})
    {
        const auto decision = decideLink(href, kDoc, kRoot);
        QVERIFY2(decision.kind == LinkKind::Refused, qPrintable(href + " passou"));
        QVERIFY(!decision.reason.isEmpty());
    }
}

void TestMarkdownPolicy::a_similar_prefix_is_not_the_same_project()
{
    // `/casa/projeto` nao pode autorizar `/casa/projeto-de-outro`: a
    // comparacao e' por componente de caminho, nao por prefixo de texto.
    QVERIFY(!insideRoot(QStringLiteral("/casa/projeto-de-outro/a.md"), kRoot));
    QVERIFY(insideRoot(QStringLiteral("/casa/projeto/a.md"), kRoot));
    QVERIFY(insideRoot(kRoot, kRoot));
    // Dois `..` a partir de `/casa/projeto/docs` chegam a `/casa`: e' dali que
    // um vizinho de nome parecido e' alcancavel. UM so' ainda esta' dentro.
    const auto sibling = decideLink(QStringLiteral("../../projeto-de-outro/a.md"), kDoc, kRoot);
    QCOMPARE(sibling.kind, LinkKind::Refused);
    const auto stillInside = decideLink(QStringLiteral("../outra/a.md"), kDoc, kRoot);
    QCOMPARE(stillInside.kind, LinkKind::LocalFile);
    QCOMPARE(stillInside.target, QStringLiteral("/casa/projeto/outra/a.md"));
}

void TestMarkdownPolicy::web_opens_outside_and_only_http_and_https()
{
    const auto decision = decideLink(QStringLiteral("https://example.org/x"), kDoc, kRoot);
    QCOMPARE(decision.kind, LinkKind::Web);
    QCOMPARE(decideLink(QStringLiteral("http://example.org"), kDoc, kRoot).kind, LinkKind::Web);
}

void TestMarkdownPolicy::an_unknown_scheme_is_refused_by_name()
{
    // Lista de PERMITIDOS: o que nao esta' nela e' recusado, e o motivo diz
    // qual esquema era — recusar em silencio seria o defeito de sempre.
    for (const QString& href :
         {QStringLiteral("javascript:alert(1)"),
          QStringLiteral("data:text/html,<script>x</script>"), QStringLiteral("ftp://servidor/x"),
          QStringLiteral("smb://maquina/compartilhado")})
    {
        const auto decision = decideLink(href, kDoc, kRoot);
        QVERIFY2(decision.kind == LinkKind::Refused, qPrintable(href + " passou"));
        QVERIFY(!decision.reason.isEmpty());
    }
}

void TestMarkdownPolicy::with_no_project_open_nothing_local_opens()
{
    // Sem projeto nao ha' escopo que autorize, e o preview de um `.md` solto
    // nao vira leitor de disco.
    const auto decision = decideLink(QStringLiteral("outro.md"), kDoc, QString());
    QCOMPARE(decision.kind, LinkKind::Refused);
    const auto image = decideResource(QStringLiteral("a.png"), kDoc, QString(), false);
    QVERIFY(!image.allowed);
    // Ancora e web continuam funcionando: nenhuma das duas toca no disco.
    QCOMPARE(decideLink(QStringLiteral("#topo"), kDoc, QString()).kind, LinkKind::Anchor);
    QCOMPARE(decideLink(QStringLiteral("https://x.org"), kDoc, QString()).kind, LinkKind::Web);
}

void TestMarkdownPolicy::a_remote_image_is_blocked_by_default()
{
    // §4.2: abrir documentacao nao busca recurso remoto — isso e' tracking por
    // acidente, e acesso a' rede sem ninguem pedir.
    const auto blocked =
        decideResource(QStringLiteral("https://cdn.exemplo/p.png"), kDoc, kRoot, false);
    QVERIFY(!blocked.allowed);
    QVERIFY(!blocked.reason.isEmpty());
    // A preferencia futura da especificacao existe, e e' explicita.
    QVERIFY(decideResource(QStringLiteral("https://cdn.exemplo/p.png"), kDoc, kRoot, true).allowed);
}

void TestMarkdownPolicy::a_local_image_outside_the_project_is_not_read()
{
    QVERIFY(decideResource(QStringLiteral("img/diagrama.png"), kDoc, kRoot, false).allowed);
    QCOMPARE(decideResource(QStringLiteral("img/diagrama.png"), kDoc, kRoot, false).path,
             QStringLiteral("/casa/projeto/docs/img/diagrama.png"));
    QVERIFY(!decideResource(QStringLiteral("../../.ssh/id_rsa"), kDoc, kRoot, false).allowed);
    QVERIFY(!decideResource(QStringLiteral("file:///etc/shadow"), kDoc, kRoot, false).allowed);
}

void TestMarkdownPolicy::an_anchor_in_a_file_link_is_not_part_of_the_path()
{
    const auto decision = decideLink(QStringLiteral("outro.md#secao"), kDoc, kRoot);
    QCOMPARE(decision.kind, LinkKind::LocalFile);
    QCOMPARE(decision.target, QStringLiteral("/casa/projeto/docs/outro.md"));
}

void TestMarkdownPolicy::a_refused_image_never_reaches_the_renderer()
{
    // Tirar a imagem da TELA depois do render e' tarde demais: quando o
    // documento ja' foi montado, o carregador do Qt Quick ja' ABRIU o arquivo.
    // A §10 pede que a imagem fora do escopo nao seja LIDA.
    const auto refuse = [](const QString& target) {
        return target.startsWith(QStringLiteral("..")) ? QStringLiteral("[fora]") : QString();
    };
    QCOMPARE(rewriteRefusedImages(QStringLiteral("antes ![x](../fuga.png) depois"), refuse),
             QStringLiteral("antes [fora] depois"));
    // A permitida fica INTACTA, com titulo e tudo.
    const QString kept = QStringLiteral(R"(![ok](img/a.png "titulo"))");
    QCOMPARE(rewriteRefusedImages(kept, refuse), kept);
}

void TestMarkdownPolicy::a_code_fence_is_content_and_is_not_rewritten()
{
    // Um exemplo de Markdown dentro de ``` e' o codigo que o autor quis
    // mostrar. Reescreve-lo mudaria o documento dele.
    const auto refuseAll = [](const QString&) { return QStringLiteral("[bloqueada]"); };
    const QString source = QStringLiteral("um ![a](x.png)\n```\n![b](y.png)\n```\n![c](z.png)");
    const QString expected = QStringLiteral("um [bloqueada]\n```\n![b](y.png)\n```\n[bloqueada]");
    QCOMPARE(rewriteRefusedImages(source, refuseAll), expected);
}

void TestMarkdownPolicy::several_images_on_one_line_are_each_decided()
{
    // Uma linha pode ter varias, e so' as recusadas mudam — as posicoes das
    // seguintes andam a cada substituicao.
    const auto refuseSecond = [](const QString& target) {
        return target == QStringLiteral("b.png") ? QStringLiteral("[nao]") : QString();
    };
    QCOMPARE(rewriteRefusedImages(QStringLiteral("![1](a.png) e ![2](b.png) e ![3](c.png)"),
                                  refuseSecond),
             QStringLiteral("![1](a.png) e [nao] e ![3](c.png)"));
}

QTEST_GUILESS_MAIN(TestMarkdownPolicy)

#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wctad-maybe-unsupported"
#endif
#include "tst_markdown_policy.moc"
#ifdef __clang__
#pragma clang diagnostic pop
#endif
