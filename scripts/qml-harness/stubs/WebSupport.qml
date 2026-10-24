pragma Singleton
import QtQuick

// The native binary supplies this C++ singleton (ui/src/web_support.h).
// Headless source-only tests see the shared context as off: no web view is
// ever created in a harness.
QtObject {
    readonly property bool contextShared: false
}
