import QtQuick
import KineinVectis

// A barra conserva a largura natural enquanto cabe, depois reparte o espaco
// e, abaixo do minimo legivel, deixa o ListView assumir a rolagem horizontal.
Item {
    id: root

    width: 900
    height: 80

    ListModel {
        id: files

        ListElement { name: "main.cpp"; modified: false }
        ListElement { name: "motor_control.cpp"; modified: false }
        ListElement { name: "CMakeLists.txt"; modified: true }
    }

    EditorTabsBar {
        id: tabs

        width: root.width
        filesModel: files
        fileCount: files.count
        currentIndex: 1
    }

    Component.onCompleted: {
        let failures = 0;

        if (tabs.tabsCompressed) failures += 1;
        if (tabs.naturalTabWidthFor("a.cpp") < tabs.minimumTabWidth) failures += 2;
        if (tabs.naturalTabWidthFor("um_nome_de_arquivo_extremamente_longo_que_precisa_ser_limitado.cpp")
                > tabs.maximumTabWidth) failures += 4;

        root.width = 360;
        if (!tabs.tabsCompressed) failures += 8;
        const expected = Math.floor((root.width - 2 * tabs.tabSpacing) / files.count);
        if (tabs.compactTabWidth !== expected) failures += 16;
        if (tabs.tabWidthFor("main.cpp") !== expected) failures += 32;

        root.width = 180;
        if (tabs.compactTabWidth !== tabs.minimumTabWidth) failures += 64;
        if (files.count * tabs.minimumTabWidth
                + (files.count - 1) * tabs.tabSpacing <= root.width) failures += 128;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
