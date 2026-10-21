pragma Singleton
import QtQuick

// The native binary supplies this C++ singleton. Headless source-only tests
// use its default flag; native acceptance also verifies the real registration.
QtObject {
    readonly property bool terminalGeometry: false
}
