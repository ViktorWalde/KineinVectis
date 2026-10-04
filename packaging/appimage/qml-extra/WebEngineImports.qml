// SO' PARA O EMPACOTADOR (roadmaps/59 §6.1). Nao faz parte da IDE nem e'
// compilado: a aba Web do Grafana cria a WebEngineView por
// Qt.createQmlObject, sem `import QtWebEngine` estatico (desligada, o modulo
// nem carrega). O linuxdeploy-plugin-qt so' leva os modulos QML que acha por
// import; este arquivo e' o import que ele precisa ver.
import QtQuick
import QtWebEngine
// O qmldir do QtWebEngine 6.4 declara `depends QtWebChannel`: sem ele no
// pacote, o import acima falha em execucao (achado no build de 2026-10-04).
import QtWebChannel

WebEngineView {}
