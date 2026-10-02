#include "frame_pacing_probe.h"

#include <QCoreApplication>
#include <QDebug>
#include <QElapsedTimer>
#include <QGuiApplication>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QTimer>

#include <algorithm>
#include <memory>
#include <mutex>
#include <vector>

namespace kinein {

namespace {

// Acima disto entre dois quadros, o Qt estava ocioso: nao e' quadro lento.
constexpr double kIdleGapMs = 100.0;
constexpr double kFrame60HzMs = 1000.0 / 60.0;
constexpr double kFrame120HzMs = 1000.0 / 120.0;
constexpr double kNsPerMs = 1'000'000.0;

// Os sinais do QQuickWindow podem vir da thread de render (render loop
// threaded): tudo o que eles escrevem passa pelo mutex.
struct FrameLog
{
    std::mutex lock;
    QElapsedTimer clock;
    qint64 lastSwapNs = -1;
    qint64 syncStartNs = -1;
    std::vector<double> intervalsMs;
    // O CUSTO de cada quadro: sincronizar e desenhar, sem a espera do vsync.
    // E' o que diz se o codigo cabe em 16,7 ms; o intervalo mistura espera.
    std::vector<double> costsMs;
};

double percentile(std::vector<double> sorted, double fraction)
{
    if (sorted.empty()) {
        return 0.0;
    }
    std::sort(sorted.begin(), sorted.end());
    const auto index = static_cast<std::size_t>(fraction * static_cast<double>(sorted.size() - 1));
    return sorted.at(index);
}

void summarize(const char* label, const std::vector<double>& samples)
{
    const auto over60 =
        std::count_if(samples.begin(), samples.end(), [](double ms) { return ms > kFrame60HzMs; });
    const auto over120 =
        std::count_if(samples.begin(), samples.end(), [](double ms) { return ms > kFrame120HzMs; });
    const double worst = samples.empty() ? 0.0 : *std::max_element(samples.begin(), samples.end());
    qInfo().noquote().nospace() << "KINEIN_PERF " << label << " n=" << samples.size()
                                << " median_ms=" << percentile(samples, 0.5)
                                << " p95_ms=" << percentile(samples, 0.95) << " max_ms=" << worst
                                << " over_16_7ms=" << over60 << " over_8_3ms=" << over120;
}

void report(FrameLog& log)
{
    const std::lock_guard<std::mutex> guard(log.lock);
    summarize("frame_interval", log.intervalsMs);
    summarize("frame_cost", log.costsMs);
}

} // namespace

void installFramePacingProbe(QGuiApplication& app, QQmlApplicationEngine& engine)
{
    bool ok = false;
    const int seconds = qEnvironmentVariableIntValue("KINEIN_PERF_FRAMES", &ok);
    if (!ok || seconds <= 0 || engine.rootObjects().isEmpty()) {
        return;
    }
    auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
    if (window == nullptr) {
        return;
    }
    auto log = std::make_shared<FrameLog>();
    log->clock.start();
    QObject::connect(
        window, &QQuickWindow::beforeSynchronizing, window,
        [log]() {
            const std::lock_guard<std::mutex> guard(log->lock);
            log->syncStartNs = log->clock.nsecsElapsed();
        },
        Qt::DirectConnection);
    QObject::connect(
        window, &QQuickWindow::afterRendering, window,
        [log]() {
            const std::lock_guard<std::mutex> guard(log->lock);
            if (log->syncStartNs >= 0) {
                log->costsMs.push_back(
                    static_cast<double>(log->clock.nsecsElapsed() - log->syncStartNs) / kNsPerMs);
                log->syncStartNs = -1;
            }
        },
        Qt::DirectConnection);
    QObject::connect(
        window, &QQuickWindow::frameSwapped, window,
        [log]() {
            const std::lock_guard<std::mutex> guard(log->lock);
            const qint64 now = log->clock.nsecsElapsed();
            if (log->lastSwapNs >= 0) {
                const double intervalMs = static_cast<double>(now - log->lastSwapNs) / kNsPerMs;
                if (intervalMs <= kIdleGapMs) {
                    log->intervalsMs.push_back(intervalMs);
                }
            }
            log->lastSwapNs = now;
        },
        Qt::DirectConnection);
    const bool exitAfter = qEnvironmentVariableIsSet("KINEIN_PERF_EXIT");
    QTimer::singleShot(seconds * 1000, &app, [log, exitAfter]() {
        report(*log);
        if (exitAfter) {
            QCoreApplication::quit();
        }
    });
}

} // namespace kinein
