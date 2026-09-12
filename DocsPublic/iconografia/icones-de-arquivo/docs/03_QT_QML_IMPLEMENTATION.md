# Implementação em Qt/QML

## 1. Recursos

Adicionar os diretórios ao resource system:

```cmake
qt_add_resources(kinein-vectis "kinein_file_icons"
    PREFIX "/kinein/icons"
    FILES
        icons/light/16/cmakelists.svg
        icons/light/20/cmakelists.svg
        icons/light/24/cmakelists.svg
        icons/dark/16/cmakelists.svg
        icons/dark/20/cmakelists.svg
        icons/dark/24/cmakelists.svg
        # repetir para project-config, sql e docker-yaml
)
```

## 2. Seleção de tamanho

```qml
pragma Singleton
import QtQuick

QtObject {
    readonly property int compactIconSize: 16
    readonly property int defaultIconSize: 20
    readonly property int comfortableIconSize: 24

    readonly property int compactRowHeight: 20
    readonly property int defaultRowHeight: 24
    readonly property int comfortableRowHeight: 28
}
```

## 3. Componente

```qml
import QtQuick
import QtQuick.Window

Item {
    id: root

    required property string iconName
    property string themeName: "dark"
    property int iconSize: 20

    width: iconSize
    height: iconSize

    Image {
        anchors.fill: parent

        source: "qrc:/kinein/icons/"
                + root.themeName + "/"
                + root.iconSize + "/"
                + root.iconName + ".svg"

        fillMode: Image.PreserveAspectFit
        cache: true
        asynchronous: false

        // Rasteriza o SVG no tamanho físico do monitor.
        sourceSize.width: Math.ceil(width * Screen.devicePixelRatio)
        sourceSize.height: Math.ceil(height * Screen.devicePixelRatio)
    }
}
```

## 4. Delegate da árvore

```qml
Item {
    id: row

    required property var model
    property int iconSize: 20
    property int rowHeight: 24

    implicitHeight: rowHeight

    KFileIcon {
        id: fileIcon

        anchors.left: parent.left
        anchors.leftMargin: 4
        anchors.verticalCenter: parent.verticalCenter

        iconSize: row.iconSize
        iconName: model.iconAssetName
        themeName: Theme.isDark ? "dark" : "light"
    }

    Text {
        anchors.left: fileIcon.right
        anchors.leftMargin: 6
        anchors.verticalCenter: parent.verticalCenter

        text: model.displayName
        color: Theme.treeText
        elide: Text.ElideRight
    }
}
```

## 5. Regras de nitidez

- manter `x`, `y`, `width` e `height` em valores inteiros;
- não aplicar `scale: 0.8` ou transformações fracionárias;
- selecionar diretamente o master 16, 20 ou 24;
- manter `fillMode: PreserveAspectFit`;
- evitar mudar `sourceSize` repetidamente durante animações;
- manter cache habilitado;
- não carregar o SVG original de 64 px na árvore.

## 6. Estratégia de desempenho

Para a primeira implementação:

```text
SVG de produção + Image + cache.
```

Quando a árvore tiver milhares de nós, medir:

```text
tempo até primeiro frame;
tempo de abertura de pasta;
uso de CPU durante scroll;
memória da cache;
jank de frame.
```

Se a rasterização inicial aparecer no perfil:

1. gerar PNGs 16/20/24 durante o build;
2. fornecer variantes `@2x`;
3. manter SVG como fonte;
4. carregar os bitmaps por tamanho/DPI.

`VectorImage` ou `svgtoqml` são mais adequados quando o mesmo asset precisa ser
transformado ou ampliado continuamente. Para ícones estáticos de árvore, uma
rasterização exata e reutilizada tende a ser mais simples.
