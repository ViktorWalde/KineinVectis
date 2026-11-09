import QtQuick

// AS FERRAMENTAS QUE O PROJETO EXIGE E AS QUE A MAQUINA TEM: a tabela por
// sistema de build e a leitura do `tools.detect`. Saiu do
// ProjectHealthController em 2026-11-09 (C-05), quando a faixa ganhou a
// escolha da CDB: a faixa decide o aviso, e quem sabe o que falta e' este
// objeto. Sem estado: quem pergunta passa a lista e os sistemas do momento,
// e um `onToolsListChanged` do controller nunca le uma copia atrasada.
QtObject {
    function hasTool(toolsList, toolId) {
        const tools = toolsList !== undefined && toolsList !== null
                ? toolsList : [];
        for (let i = 0; i < tools.length; i++) {
            if (tools[i].id === toolId && tools[i].status === "detected") {
                return true;
            }
        }
        return false;
    }

    function hasAnyTool(toolsList, toolIds) {
        for (let i = 0; i < toolIds.length; i++) {
            if (hasTool(toolsList, toolIds[i])) {
                return true;
            }
        }
        return false;
    }

    function requiredToolGroups(kind) {
        if (kind === "rustCargo") {
            return [["cargo"], ["rustc"], ["rust-analyzer"]];
        }
        if (kind === "cmake") {
            return [["cmake"], ["ninja"], ["clangd"], ["clangxx", "gxx"]];
        }
        // Makefile puro (P0, 2026-09-17): o bear entra como ferramenta do
        // projeto porque sem ele nao ha' compile_commands.json — e a faixa
        // diz isso antes do primeiro build mudo.
        if (kind === "make") {
            return [["make", "gmake"], ["clangd"], ["clangxx", "gxx"], ["bear"]];
        }
        // PlatformIO (0.4.0e): o `pio` constroi e gera a CDB; o clangd a le'.
        if (kind === "platformIo") {
            return [["pio"], ["clangd"]];
        }
        return [];
    }

    function requiredWorkspaceToolGroups(workspaceKind, workspaceBuildSystems) {
        const groups = [];
        const systems = workspaceBuildSystems !== undefined
                && workspaceBuildSystems !== null
                ? workspaceBuildSystems : [];
        for (let index = 0; index < systems.length; index++) {
            let systemGroups = [];
            if (systems[index] === "cargo") {
                systemGroups = requiredToolGroups("rustCargo");
            } else if (systems[index] === "cmake" || systems[index] === "platformIo") {
                systemGroups = requiredToolGroups(systems[index]);
            } else if (systems[index] === "make" && systems.indexOf("cmake") < 0) {
                // Um Makefile ao lado de um CMakeLists e' do CMake: o bear so'
                // e' exigido quando o make e' o unico build system C/C++.
                systemGroups = requiredToolGroups("make");
            }
            for (let groupIndex = 0; groupIndex < systemGroups.length;
                 groupIndex++) {
                const key = systemGroups[groupIndex].join("|");
                let duplicate = false;
                for (let existing = 0; existing < groups.length; existing++) {
                    if (groups[existing].join("|") === key) {
                        duplicate = true;
                        break;
                    }
                }
                if (!duplicate) {
                    groups.push(systemGroups[groupIndex]);
                }
            }
        }
        return groups.length > 0 ? groups : requiredToolGroups(workspaceKind);
    }

    function missingGroups(toolsList, groups) {
        const missing = [];
        for (let i = 0; i < groups.length; i++) {
            if (!hasAnyTool(toolsList, groups[i])) {
                missing.push(groups[i].join("/"));
            }
        }
        return missing;
    }
}
