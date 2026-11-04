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
        return qsTr("Ferramenta: %1").arg(executable.trim() || qsTr("automática"))
               + (language === "python" ? "" : "\n" + qsTr("Padrão/edição: %1").arg(standard.trim() || qsTr("da ferramenta")));
    }
}
