import QtQuick

EmbeddedUsbQueryController {
    id: root
    property alias devices: root.entries
    entriesKey: "devices"
    queryingText: qsTr("Consultando RP2040 em BOOTSEL…")
    unverifiedText: qsTr("Consulta RP2040 não verificada")
    completeText: qsTr("%1 RP2040 confirmado(s) pelo picotool.").arg(devices.length)
    incompleteText: qsTr("A consulta RP2040 não foi verificada. Confira a saída e tente novamente.")
    function summary(entry) {
        return qsTr("RP2040 · seletor %1 · %2 KiB de flash informados")
            .arg(entry.serialSelector).arg(entry.flashSizeBytes / 1024);
    }
}
