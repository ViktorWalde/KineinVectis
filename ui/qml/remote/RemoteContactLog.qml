import QtQuick

// O ULTIMO CONTATO com cada alvo (0.153.0, 2026-10-04), como o core guarda
// em `.kinein/remote-contacts.json`: o que a ultima sonda achou e quando.
// Filho do RemoteController. A lista de alvos diz, para TODOS e nao so' o
// escolhido, "respondeu ha' 3 min · x86_64" ou "falhou ontem".
QtObject {
    id: root

    // name -> { ok, at (segundos Unix), arch, kernel, failure }
    property var byName: ({})
    // O relogio do "ha' 3 min", UM para a lista e a Visao (as duas diziam
    // idades diferentes: a Visao lia o Date.now() uma vez so'). Quem o faz
    // andar e' a lista, enquanto esta' a vista.
    property real nowSeconds: Date.now() / 1000

    function handleList(list) {
        const map = {};
        for (const contact of list) map[contact.name] = contact;
        root.byName = map;
    }

    // A sonda que acabou de chegar, sem esperar o proximo `remote.list`.
    function record(outcome) {
        const map = Object.assign({}, root.byName);
        root.nowSeconds = Date.now() / 1000;
        map[outcome.name] = { name: outcome.name, ok: outcome.success === true, at: Math.floor(root.nowSeconds),
                              arch: outcome.arch || "", kernel: outcome.kernel || "", failure: outcome.failure || "" };
        root.byName = map;
    }

    function forget(name) {
        const map = Object.assign({}, root.byName);
        delete map[name];
        root.byName = map;
    }

    // "ok" | "fail" | "" (nunca sondado)
    function state(name) {
        const contact = root.byName[name];
        return contact === undefined ? "" : (contact.ok ? "ok" : "fail");
    }

    // "agora", "há 3 min", "há 2 h", "ontem", "há 5 dias"
    function age(at, nowSeconds) {
        const seconds = Math.max(0, nowSeconds - at);
        if (seconds < 60) return qsTr("agora");
        if (seconds < 3600) return qsTr("há %1 min").arg(Math.floor(seconds / 60));
        if (seconds < 86400) return qsTr("há %1 h").arg(Math.floor(seconds / 3600));
        if (seconds < 172800) return qsTr("ontem");
        return qsTr("há %1 dias").arg(Math.floor(seconds / 86400));
    }

    function describe(name, nowSeconds) {
        const contact = root.byName[name];
        if (contact === undefined) return qsTr("nunca sondado");
        const when = root.age(contact.at, nowSeconds);
        return contact.ok ? qsTr("respondeu %1").arg(when) + (contact.arch ? " · " + contact.arch : "")
                          : qsTr("não respondeu %1").arg(when);
    }
}
