// Toolchain: o que a UI mostra e o que ela pede (roadmap 30, etapa 5).
//
// Por que existe: o `ToolchainController` traduz a resposta do core no rotulo
// da barra de status e no estado de cada linha do menu. Se essa traducao
// quebrar, nao ha compilador que reclame — a IDE so passa a dizer "automatica"
// para um projeto que tem escolha fixada, ou a marcar o item errado com ✓.
// E' a mesma classe do `cdbStale`: dado medido que nao chega a tela.
//
// O outro alvo e o CONTRATO do "automatico": a UI manda id VAZIO para liberar
// a escolha, e o C++ traduz isso em campo AUSENTE. Mandar a string vazia como
// id faria o core recusar ("candidato desconhecido") — e o usuario ficaria
// preso na escolha que fez.
import QtQuick
import "../../ui/qml/toolchain"

Item {
    id: root

    property var pedidos: []
    property var kits: []
    property int consultas: 0

    ToolchainController {
        id: controller

        onGetRequested: root.consultas += 1
        onSetRequested: function (role, id) {
            root.pedidos.push({ role: role, id: id });
        }
        onSetKitRequested: function (preset, sysroot, targetTriple, chip) {
            root.kits.push({ preset: preset, sysroot: sysroot,
                             targetTriple: targetTriple, chip: chip });
        }
    }

    // O SHAPE E' O DO CORE, campo por campo. Desde 2026-09-04 a resposta traz
    // `effectiveId` e `automatic`: sem escolha fixada, o core PEGA o primeiro
    // candidato e diz que a escolha foi dele. Fixture sem esses campos testa
    // um protocolo que nao existe mais — foi assim que o `fd` quebrou a busca
    // com o gate verde (DocsPublic/roadmaps/39 §8.2).
    function selecoes() {
        return [
            { role: "cxxCompiler", id: "gxx", effectiveId: "gxx",
              resolvedPath: "/usr/bin/g++", automatic: false },
            { role: "cCompiler", effectiveId: "clang", automatic: true },
            { role: "generator", id: "Ninja", effectiveId: "Ninja",
              resolvedPath: "/usr/bin/ninja", automatic: false },
            { role: "cmake", automatic: true },
            { role: "cargo", automatic: true }
        ];
    }

    function candidatos() {
        return [
            { role: "cxxCompiler", id: "clangxx", label: "Clang++", path: "/usr/bin/clang++" },
            { role: "cxxCompiler", id: "gxx", label: "G++", path: "/usr/bin/g++" },
            { role: "cCompiler", id: "clang", label: "Clang", path: "/usr/bin/clang" },
            { role: "generator", id: "Ninja", label: "Ninja", path: "/usr/bin/ninja" }
        ];
    }

    Component.onCompleted: {
        let failures = 0;

        // Workspace novo: nada fixado, e a barra diz isso.
        controller.workspaceRoot = "/tmp/projeto";
        if (root.consultas !== 1) failures += 1;
        if (controller.summary() !== "nenhuma detectada") failures += 2;

        controller.handleResolved(root.selecoes(), root.candidatos());

        // O resumo mostra o que vai ser USADO — C++ e gerador.
        if (controller.summary() !== "G++ · Ninja") failures += 4;
        if (controller.labelFor("cxxCompiler") !== "G++") failures += 8;
        // ESCOLHA AUTOMATICA NAO E' MAIS "nao sei": ela diz QUAL, e diz que
        // foi o core que escolheu. Sem isso o autor nao tem como discordar de
        // uma decisao que nem sabe que foi tomada.
        if (controller.labelFor("cCompiler") !== "Clang · automático") failures += 16;
        if (!controller.isAutomatic("cCompiler")) failures += 131072;
        if (controller.isAutomatic("cxxCompiler")) failures += 262144;

        // A lista de um papel traz so os candidatos DAQUELE papel.
        const opcoesCxx = controller.candidatesFor("cxxCompiler");
        if (opcoesCxx.length !== 2) failures += 32;
        if (opcoesCxx[0].id !== "clangxx") failures += 64;

        // Escolher manda o id; LIBERAR manda vazio — e' o contrato com o C++,
        // que so entao omite o campo e o core entende "automatico".
        controller.choose("cCompiler", "clang");
        controller.choose("cxxCompiler", "");
        if (root.pedidos.length !== 2) {
            failures += 128;
        } else {
            if (root.pedidos[0].role !== "cCompiler" || root.pedidos[0].id !== "clang") failures += 256;
            if (root.pedidos[1].role !== "cxxCompiler" || root.pedidos[1].id !== "") failures += 512;
        }

        // O CHIP do kit chega e volta (2026-09-11): ele existia no protocolo e
        // a ponte o omitia. `applyKit` manda os TRES campos, chip inclusive.
        controller.handleResolved([], [], "nucleo", "/opt/sysroot",
                                  "thumbv7em-none-eabihf", "STM32F401CC", "");
        if (controller.chip !== "STM32F401CC") failures += 524288;
        if (controller.preset !== "nucleo") failures += 1048576;
        controller.applyKit("/opt/sysroot", "thumbv7em-none-eabihf", "STM32F401CC");
        if (root.kits.length !== 1 || root.kits[0].chip !== "STM32F401CC"
                || root.kits[0].preset !== "nucleo") failures += 2097152;
        // `toolchain.setKit` recusado e' erro DESTE menu.
        controller.handleFailed("toolchain.setKit", "chip desconhecido");
        if (controller.errorText !== "chip desconhecido") failures += 4194304;

        // O alvo Rust do kit (integracoes/39): sem rustup nada a dizer; com
        // rustup e o alvo fora da lista, o comando exato; instalado, silencio.
        controller.handleAdvice("", [], false);
        if (controller.rustTargetHint() !== "") failures += 8388608;
        controller.handleAdvice("", ["x86_64-unknown-linux-gnu"], true);
        if (controller.rustTargetHint() !== "alvo Rust thumbv7em-none-eabihf não instalado: rustup target add thumbv7em-none-eabihf") failures += 16777216;
        controller.handleAdvice("", ["x86_64-unknown-linux-gnu", "thumbv7em-none-eabihf"], true);
        if (controller.rustTargetHint() !== "") failures += 33554432;
        // A dica de sysroot chega como veio; ausente = vazia.
        controller.handleAdvice("o compilador cross nao traz o sistema alvo", [], false);
        if (controller.sysrootHint.indexOf("sistema alvo") < 0) failures += 67108864;
        controller.handleAdvice(undefined, undefined, undefined);
        if (controller.sysrootHint !== "" || controller.rustTargetHint() !== "") failures += 134217728;

        // Escolha cujo binario sumiu da maquina: a UI DIZ que sumiu em vez de
        // mostrar um rotulo bonito. O core tambem para de fixar o caminho.
        controller.handleResolved(
            [{ role: "cxxCompiler", id: "gxx" }],
            [{ role: "cxxCompiler", id: "clangxx", label: "Clang++" }]);
        if (controller.labelFor("cxxCompiler") !== "gxx (ausente)") failures += 1024;

        // Erro de OUTRO dominio nao acende o erro deste menu.
        controller.handleFailed("git.status", "nada a ver");
        if (controller.errorText !== "") failures += 2048;
        controller.handleFailed("toolchain.set", "g++ nao foi detectado");
        if (controller.errorText !== "g++ nao foi detectado") failures += 4096;

        // Trocar de workspace ESQUECE tudo: toolchain e por projeto.
        controller.workspaceRoot = "/tmp/outro";
        if (controller.selections.length !== 0) failures += 8192;
        if (controller.errorText !== "") failures += 16384;
        if (controller.summary() !== "nenhuma detectada") failures += 32768;
        if (root.consultas !== 2) failures += 65536;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
