import QtQuick

// Composition keeps the existing public DFU owner and adds Pico within its domain.
Item {
    id: root
    visible: false
    readonly property alias dfu: dfuController
    readonly property alias pico: picoController
    readonly property string verdict: picoController.busy || picoController.measured || picoController.errorText !== ""
        ? picoController.verdict : dfuController.verdict
    EmbeddedDfuController { id: dfuController }
    EmbeddedPicoController { id: picoController }
    function clear() { dfuController.clear(); picoController.clear(); }
    function disconnected() { dfuController.disconnected(); picoController.disconnected(); }
}
