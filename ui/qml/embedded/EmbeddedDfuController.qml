import QtQuick

// DFU presentation; query identity and lifecycle have one shared owner.
EmbeddedUsbQueryController {
    id: root
    property alias interfaces: root.entries
    entriesKey: "interfaces"
    queryingText: qsTr("Consultando USB DFU…")
    unverifiedText: qsTr("Consulta USB DFU não verificada")
    completeText: qsTr("%1 alternativa(s) USB DFU listada(s).").arg(interfaces.length)
    incompleteText: qsTr("A consulta USB DFU não foi verificada. Confira a saída e tente novamente.")

    function isDfu(entry) { return !!entry && entry.mode === "dfu"; }

    function summary(entry) {
        const mode = isDfu(entry) ? qsTr("DFU") : qsTr("Runtime");
        return qsTr("%1 · %2:%3 · configuração %4 · interface %5 · alternativa %6")
            .arg(mode).arg(entry.vid).arg(entry.pid).arg(entry.configuration).arg(entry.interface).arg(entry.alternate);
    }
}
