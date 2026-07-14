#include "core_client.h"

namespace kinein {

bool CoreClient::isBuilding() const
{
    return m_building;
}

void CoreClient::setBuilding(bool building)
{
    if (m_building == building) {
        return;
    }
    m_building = building;
    emit buildingChanged();
}

bool CoreClient::isTesting() const
{
    return m_testing;
}

void CoreClient::setTesting(bool testing)
{
    if (m_testing == testing) {
        return;
    }
    m_testing = testing;
    emit testingChanged();
}

bool CoreClient::isAnalyzing() const
{
    return m_analyzing;
}

void CoreClient::setAnalyzing(bool analyzing)
{
    if (m_analyzing == analyzing) {
        return;
    }
    m_analyzing = analyzing;
    emit analyzingChanged();
}

bool CoreClient::isRunning() const
{
    return m_running;
}

void CoreClient::setRunning(bool running)
{
    if (m_running == running) {
        return;
    }
    m_running = running;
    emit runningChanged();
}

bool CoreClient::isDebugging() const
{
    return m_debugging;
}

void CoreClient::setDebugging(bool debugging)
{
    if (m_debugging == debugging) {
        return;
    }
    m_debugging = debugging;
    emit debuggingChanged();
}

bool CoreClient::isTerminalActive() const
{
    return m_terminalActive;
}

void CoreClient::setTerminalActive(bool active)
{
    if (m_terminalActive == active) {
        return;
    }
    m_terminalActive = active;
    emit terminalActiveChanged();
}

bool CoreClient::isScanningEnvironment() const
{
    return m_scanningEnvironment;
}

void CoreClient::setScanningEnvironment(bool scanning)
{
    if (m_scanningEnvironment == scanning) {
        return;
    }
    m_scanningEnvironment = scanning;
    emit scanningEnvironmentChanged();
}

bool CoreClient::isRecovering() const
{
    return m_recovering;
}

void CoreClient::setRecovering(bool recovering)
{
    if (m_recovering == recovering) {
        return;
    }
    m_recovering = recovering;
    emit recoveringChanged();
}

} // namespace kinein
