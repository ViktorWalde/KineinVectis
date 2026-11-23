// O rascunho tem prazo máximo (INF-26, 62 §3.4).
//
// O defeito que isto previne: cada tecla reiniciava o temporizador do
// rascunho, e quem digitava sem uma pausa de 1,5 s não tinha nada gravado.
// Medido em 2026-11-15: 100 s de digitação e `kill -9` logo depois, 0 teclas
// recuperadas em 3 de 3. A correção é um segundo temporizador que a primeira
// mudança liga e as seguintes não reiniciam.
//
// O teste carrega o `EditorPersistenceController.qml` de verdade, com pausa e
// prazo encurtados, e digita mais rápido que a pausa: só o prazo pode gravar.
// O tempo é o que se mede aqui, então o teste usa o relógio; as folgas cobrem
// a variação de agenda da máquina, não a regra.
#include <QElapsedTimer>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QRegularExpression>
#include <QSignalSpy>
#include <QtTest>

#include <algorithm>
#include <memory>
#include <vector>

namespace {

constexpr int kPauseMs = 300;
constexpr int kMaxWaitMs = 400;
constexpr int kKeyEveryMs = 50;  // bem abaixo da pausa: o debounce nunca dispara
constexpr int kPollMs = 10;      // resolução com que o teste vê cada gravação
constexpr int kTypingMs = 2000;
constexpr int kSchedulingMarginMs = 250;    // agenda da máquina, ASan incluso
constexpr int kSettleMs = 1000;
// Com o prazo, cerca de kTypingMs / kMaxWaitMs gravações; sem ele, zero.
constexpr qsizetype kMinSavesWhileTyping = kTypingMs / (kMaxWaitMs + kSchedulingMarginMs);

// Um controlador com pontes falsas: o buffer está sempre sujo, e o texto é o
// número da última tecla, para cada gravação dizer até onde chegou.
QByteArray probeSource()
{
    return QByteArrayLiteral(R"(
import QtQuick
EditorPersistenceController {
    id: probe
    property int edits: 0
    autoSaveEnabled: false
    surfaceBridge: QtObject {
        readonly property QtObject editorSurface: null
        function ready() { return true; }
        function text() { return String(probe.edits); }
    }
    documentController: QtObject {
        readonly property bool currentReadOnly: false
        function currentFilePath() { return "/kinein-teste/a.cpp"; }
        function currentIsModified() { return true; }
    }
}
)");
}

struct Edit
{
    int number;
    qint64 atMs;
};

struct Save
{
    qint64 atMs;
    int lastEdit;
};

// Acompanha o sinal `draftSaveRequested` e anota quando cada gravação foi vista.
class SaveLog
{
public:
    SaveLog(QObject& controller, const QElapsedTimer& clock)
        : spy_(&controller, SIGNAL(draftSaveRequested(QString, QString))), clock_(clock)
    {
    }

    [[nodiscard]] bool valid() const { return spy_.isValid(); }
    [[nodiscard]] const std::vector<Save>& saves() const { return saves_; }

    // Espera `ms` processando eventos, e anota as gravações novas.
    void waitCollecting(int ms)
    {
        QElapsedTimer waited;
        waited.start();
        while (waited.elapsed() < ms) {
            QTest::qWait(kPollMs);
            for (; seen_ < spy_.count(); ++seen_) {
                const int lastEdit = spy_.at(seen_).at(1).toString().toInt();
                saves_.push_back(Save{.atMs = clock_.elapsed(), .lastEdit = lastEdit});
            }
        }
    }

private:
    QSignalSpy spy_;
    const QElapsedTimer& clock_;
    qsizetype seen_ = 0;
    std::vector<Save> saves_;
};

// Uma tecla a cada kKeyEveryMs, por kTypingMs. Devolve quando parou.
qint64 typeWithoutPause(QObject& controller, const QElapsedTimer& clock, SaveLog& log,
                        std::vector<Edit>& edits)
{
    for (int number = 1; clock.elapsed() < kTypingMs; ++number) {
        controller.setProperty("edits", number);
        QMetaObject::invokeMethod(&controller, "scheduleDraftSave");
        edits.push_back(Edit{.number = number, .atMs = clock.elapsed()});
        log.waitCollecting(kKeyEveryMs);
    }
    return clock.elapsed();
}

// A primeira tecla que não chegou a uma gravação dentro do prazo, ou vazio.
QString firstLateEdit(const std::vector<Edit>& edits, const std::vector<Save>& saves)
{
    for (const Edit& edit : edits) {
        const auto covering = std::ranges::find_if(
            saves, [&](const Save& save) { return save.lastEdit >= edit.number; });
        if (covering == saves.end()) {
            return QStringLiteral("a tecla %1 nunca foi gravada").arg(edit.number);
        }
        const qint64 waitedMs = covering->atMs - edit.atMs;
        if (waitedMs > kMaxWaitMs + kSchedulingMarginMs) {
            return QStringLiteral("a tecla %1 esperou %2 ms").arg(edit.number).arg(waitedMs);
        }
    }
    return {};
}

} // namespace

class TestEditorDraftDeadline : public QObject
{
    Q_OBJECT

    QQmlEngine engine_;
    std::unique_ptr<QObject> controller_;

private slots:
    void init()
    {
        // Aviso de QML (tipo não achado, propriedade inválida) reprova.
        QTest::failOnWarning(QRegularExpression(QStringLiteral(".*")));

        QQmlComponent component(&engine_);
        // A URL na pasta do editor resolve o tipo pelo import implícito do
        // diretório: é o mesmo arquivo que o aplicativo carrega. A variável
        // de ambiente aponta para uma cópia mutada, na medida dos mutantes.
        const QString editorDir = qEnvironmentVariable(
            "KINEIN_QML_EDITOR_DIR", QStringLiteral(KINEIN_QML_EDITOR_DIR));
        component.setData(probeSource(), QUrl::fromLocalFile(editorDir + QStringLiteral("/probe.qml")));
        controller_.reset(component.create());
        QVERIFY2(controller_ != nullptr, qPrintable(component.errorString()));
        controller_->setProperty("draftPauseMs", kPauseMs);
        controller_->setProperty("draftMaxWaitMs", kMaxWaitMs);
    }

    void cleanup() { controller_.reset(); }

    void typing_without_pause_saves_every_edit_within_the_deadline()
    {
        QElapsedTimer clock;
        SaveLog log(*controller_, clock);
        QVERIFY(log.valid());

        std::vector<Edit> edits;
        clock.start();
        const qint64 typingEndedMs = typeWithoutPause(*controller_, clock, log, edits);
        log.waitCollecting(kSettleMs);

        // Sem o prazo, nada grava enquanto as teclas vêm mais rápido que a pausa.
        const auto whileTyping = std::ranges::count_if(
            log.saves(), [&](const Save& save) { return save.atMs <= typingEndedMs; });
        QVERIFY2(whileTyping >= kMinSavesWhileTyping,
                 qPrintable(QStringLiteral("%1 gravações durante %2 ms de digitação")
                                .arg(whileTyping)
                                .arg(typingEndedMs)));

        // Cada tecla chega a uma gravação em até o prazo, mais a folga.
        const QString late = firstLateEdit(edits, log.saves());
        QVERIFY2(late.isEmpty(), qPrintable(late));
    }

    void a_pause_saves_once_before_the_deadline()
    {
        QElapsedTimer clock;
        SaveLog log(*controller_, clock);
        QVERIFY(log.valid());

        clock.start();
        controller_->setProperty("edits", 1);
        QVERIFY(QMetaObject::invokeMethod(controller_.get(), "scheduleDraftSave"));
        log.waitCollecting(kSettleMs);

        // A pausa grava e desliga o prazo: uma gravação só, não duas.
        QCOMPARE(log.saves().size(), std::size_t{1});
        const qint64 savedAtMs = log.saves().front().atMs;
        QVERIFY2(savedAtMs < kMaxWaitMs, qPrintable(QStringLiteral("gravou aos %1 ms").arg(savedAtMs)));
    }
};

QTEST_MAIN(TestEditorDraftDeadline)

// O moc do Qt 6.12 usa CTAD; mesma fronteira dos demais testes (52 §14.50).
#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wctad-maybe-unsupported"
#endif
#include "tst_editor_draft_deadline.moc"
#ifdef __clang__
#pragma clang diagnostic pop
#endif
