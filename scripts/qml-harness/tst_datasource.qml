import QtQuick
// Carrega o DataSourceController.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/datasource"

// A INVARIANTE DE SEGREDO da UI, exercitada.
//
// POR QUE ESTE TESTE EXISTE (2026-09-04). O core garante que a senha nunca vai
// para o disco — ha' teste para isso, e o perfil nem tem campo. Mas a senha
// EXISTE na UI, em memoria, durante a sessao. O jeito de ela vazar aqui nao e'
// gravar arquivo: e' sobreviver a uma troca de contexto e ser enviada para o
// perfil ERRADO, ou continuar em memoria depois de o painel fechar.
//
// Nenhuma dessas falhas produz erro. As duas produzem uma senha indo para um
// servidor que nao e' o dela.
Item {
    id: root
    width: 100; height: 100

    property var enviados: []

    Item {
        visible: false

        DataSourceController {
            id: fontes

            onSaveRequested: profile => root.enviados.push(profile)
            onTestRequested: (name, password) =>
                root.enviados.push({ name: name, password: password })
        }
    }

    Component.onCompleted: {
        let f = 0;

        // 1) O padrao e' o caso comum do autor: socket unix, sem senha.
        //    Se isto virar "prompt", a IDE volta a inventar um obstaculo que o
        //    servidor nao impoe (DocsPublic/seguranca/40 §7).
        const padrao = fontes.emptyDraft();
        if (padrao.secretSource !== "automatic") f += 1;
        if (padrao.host.charAt(0) !== "/") f += 2;

        // 2) A senha da sessao NAO sobrevive a troca de perfil.
        fontes.profiles = [
            { name: "a", host: "/tmp/s", port: 5432, database: "d", user: "u",
              secretSource: "prompt", secretVariable: "" },
            { name: "b", host: "127.0.0.1", port: 5432, database: "d", user: "u",
              secretSource: "prompt", secretVariable: "" }
        ];
        fontes.select("a");
        fontes.sessionPassword = "segredo-de-a";
        fontes.select("b");
        if (fontes.sessionPassword !== "") f += 4;

        // 3) Nem a fechar o painel.
        fontes.open();
        fontes.sessionPassword = "segredo-vivo";
        fontes.close();
        if (fontes.sessionPassword !== "") f += 8;

        // 4) Nem a trocar de projeto.
        fontes.sessionPassword = "segredo-do-projeto";
        fontes.workspaceRoot = "/tmp/outro-projeto";
        if (fontes.sessionPassword !== "") f += 16;
        if (fontes.profiles.length !== 0) f += 32;

        // 5) O que vai para o `save` tem SO' os campos do protocolo. O perfil
        //    tem `deny_unknown_fields`: um campo a mais e' RECUSADO, e um
        //    campo `password` a mais seria a falha que tudo isto evita.
        root.enviados = [];
        fontes.profiles = [];
        fontes.draft = {
            name: "c", host: "/tmp/s", port: 5432, database: "d", user: "u",
            secretSource: "automatic", secretVariable: "",
            password: "NAO PODE IR", extra: 1
        };
        fontes.save();
        const enviado = root.enviados[0];
        if (enviado === undefined) {
            f += 64;
        } else {
            if (enviado.password !== undefined) f += 128;
            if (enviado.extra !== undefined) f += 256;
            if (Object.keys(enviado).length !== 8) f += 512;
        }

        // 6) O veredito vem por CAMPO. A mensagem do servidor e' localizada, e
        //    ler ela para decidir acoplaria a UI ao idioma do banco.
        fontes.handleTested("c", false,
                            "", "FATAL: autenticação do tipo senha falhou", true);
        if (!fontes.secretRequired || fontes.testing) f += 1024;
        fontes.handleTested("c", true, "PostgreSQL 18.6", "", false);
        if (!fontes.testOk || fontes.secretRequired) f += 2048;
        if (fontes.serverVersion !== "PostgreSQL 18.6") f += 4096;

        // 7) Testar leva a senha da sessao junto — e' o unico envio dela.
        root.enviados = [];
        fontes.sessionPassword = "certa";
        fontes.test();
        if (root.enviados[0].password !== "certa") f += 8192;
        if (!fontes.testing) f += 16384;

        // O codigo de saida de um processo tem 8 BITS: o mask vai para a SAIDA
        // (onde nao trunca) e o exit so diz passou/falhou.
        if (f !== 0) console.error("FALHAS bitmask=" + f);
        Qt.exit(f === 0 ? 0 : 1);
    }
}
