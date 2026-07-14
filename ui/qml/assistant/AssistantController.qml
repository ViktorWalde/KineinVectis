import QtQuick

// Estado do AI CLI Bridge. A sessão continua sendo um PTY do domínio terminal;
// este controller só separa a experiência visual e a escolha do perfil.
Item {
    id: root

    property alias profilesModel: profilesListModel
    property string selectedProfileId: "claude"
    property string sessionId: ""
    property string activeProfileId: ""
    property string activeProfileName: ""
    property string activeCommand: ""
    property var terminalRender: ({})
    property string errorText: ""
    property bool loading: false

    signal profilesRequested()
    signal terminalOpenRequested(string profileId)
    signal terminalInputRequested(string id, string data)
    signal terminalResizeRequested(string id, int cols, int rows)
    signal terminalScrollRequested(string id, int offset)
    signal terminalCloseRequested(string id)
    signal profilePreferenceRequested(string profileId)

    visible: false

    ListModel {
        id: profilesListModel
    }

    function initialize() {
        profilesRequested();
    }

    function profileIndex(id) {
        for (let i = 0; i < profilesListModel.count; i++) {
            if (profilesListModel.get(i).profileId === id) return i;
        }
        return -1;
    }

    function profileAvailable(id) {
        const index = profileIndex(id);
        return index >= 0 && profilesListModel.get(index).available;
    }

    function handleProfiles(profiles, defaultProfile) {
        profilesListModel.clear();
        for (let i = 0; i < profiles.length; i++) {
            profilesListModel.append({
                profileId: profiles[i].id,
                name: profiles[i].name,
                command: profiles[i].command,
                available: profiles[i].available === true
            });
        }
        if (profileAvailable(defaultProfile)) {
            selectedProfileId = defaultProfile;
        } else if (profileAvailable("claude")) {
            selectedProfileId = "claude";
        } else if (profileAvailable("codex")) {
            selectedProfileId = "codex";
        } else {
            selectedProfileId = defaultProfile === "codex" ? "codex" : "claude";
        }
        errorText = "";
    }

    function selectProfile(id) {
        selectedProfileId = id;
        errorText = "";
    }

    function startSelectedProfile() {
        if (!profileAvailable(selectedProfileId)) {
            errorText = qsTr("A CLI selecionada precisa estar instalada e disponível no PATH.");
            return;
        }
        loading = true;
        errorText = "";
        terminalOpenRequested(selectedProfileId);
    }

    function handleTerminalOpened(id, profileId, name, command) {
        sessionId = id;
        activeProfileId = profileId;
        activeProfileName = name;
        activeCommand = command;
        terminalRender = ({});
        loading = false;
        errorText = "";
        profilePreferenceRequested(profileId);
    }

    function sendKey(data) {
        if (sessionId !== "") terminalInputRequested(sessionId, data);
    }

    function resizeTerminal(cols, rows) {
        if (sessionId !== "") terminalResizeRequested(sessionId, cols, rows);
    }

    function scrollTerminal(offset) {
        if (sessionId !== "") terminalScrollRequested(sessionId, offset);
    }

    function exitSession() {
        const id = sessionId;
        clearSession();
        if (id !== "") terminalCloseRequested(id);
    }

    function switchProfile() {
        exitSession();
        profilesRequested();
    }

    function clearSession() {
        sessionId = "";
        activeProfileId = "";
        activeProfileName = "";
        activeCommand = "";
        terminalRender = ({});
        loading = false;
    }

    function handleTerminalRender(render) {
        if (render && render.id === sessionId) terminalRender = render;
    }

    function handleTerminalClosed(id) {
        if (id === sessionId) clearSession();
    }

    function handleRequestFailed(method, message) {
        if (method === "aiBridge.profiles"
                || method === "aiBridge.terminal.open") {
            loading = false;
            errorText = message;
        }
    }
}
