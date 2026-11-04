import QtQuick

QtObject {
    id: root
    property var environments: []
    property string executable: ""
    property string standard: ""
    property bool loading: false
    property string errorText: ""
    signal environmentsRequested()

    function reset() { executable = ""; standard = ""; errorText = ""; }
    function refresh() { loading = true; errorText = ""; environmentsRequested(); }
    function resolved(entries) { environments = entries; loading = false; errorText = ""; }
    function failed(message) { loading = false; errorText = message; }
    function forLanguage(language) { return environments.filter(entry => entry.language === language); }
    function payload() {
        const result = {};
        if (executable.trim() !== "") result.executable = executable.trim();
        if (standard.trim() !== "") result.standard = standard.trim();
        return result;
    }
    function preview(language) {
        if (language === "" || language === "empty") return "";
        const tool = qsTr("Ferramenta: %1").arg(executable.trim() || qsTr("automática"));
        if (language === "python") return tool;
        const value = standard.trim();
        if (language === "c") {
            return tool + "\n" + (value === "" ? qsTr("Padrão C: da ferramenta (sem revisão imposta)")
                : /^\d+$/.test(value) ? qsTr("Padrão CMake: %1 (ISO obrigatório)").arg(value)
                : qsTr("Dialeto C: -std=%1 (literal)").arg(value));
        }
        return tool + "\n" + qsTr("Padrão/edição: %1").arg(value || qsTr("da ferramenta"));
    }
}
