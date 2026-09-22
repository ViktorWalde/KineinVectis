# Implementação em Qt/QML

## 1. Recursos

Adicionar os masters PNG de produção ao resource system:

```cmake
qt_add_qml_module(kinein-vectis
    # ...
    RESOURCES
        assets/icons/tree/file-c.png
        assets/icons/tree/file-h.png
        assets/icons/tree/file-hpp.png
        assets/icons/tree/file-cmakelists.png
        assets/icons/tree/file-rust.png
        assets/icons/tree/file-python.png
        assets/icons/tree/file-yaml.png
        assets/icons/tree/file-sql.png
        assets/icons/tree/file-markdown.png
        assets/icons/tree/file-docker.png
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
Item {
    id: root

    required property string iconName
    property int iconSize: 20

    width: iconSize
    height: iconSize

    Image {
        anchors.fill: parent

        source: "qrc:/KineinVectis/assets/icons/tree/"
                + root.iconName + ".png"

        fillMode: Image.PreserveAspectFit
        cache: true
        asynchronous: false

        smooth: true
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
- exibir o master PNG de 64 px em 16 ou 20 px inteiros;
- manter `fillMode: PreserveAspectFit`;
- manter cache habilitado;
- não carregar os PNGs originais de mais de 1200 px na árvore.

## 6. Estratégia de desempenho

Para a primeira implementação:

```text
PNG de produção de 64 px + `Image` + cache.
```

Quando a árvore tiver milhares de nós, medir:

```text
tempo até primeiro frame;
tempo de abertura de pasta;
uso de CPU durante scroll;
memória da cache;
jank de frame.
```

Se o redimensionamento aparecer no perfil, gerar variantes dedicadas de 20,
40 e 60 px para os fatores de escala 1x, 2x e 3x. Os originais grandes ficam
como entrada de design e não entram no recurso Qt.
