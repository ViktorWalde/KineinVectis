# O QT DO PROJETO (2026-10-24; roadmaps/40 §5 "qt do projeto", 52 §14.49).
#
# O autor decidiu o 6.12, e teste e ambiente de teste usam a mesma versao do
# frontend. Ate' esta data o CMake pedia `Qt6 6.4` e aceitava o primeiro Qt que
# achasse: na volta ao Ubuntu (2026-10-15) o build passou do 6.12 do Arch para
# o 6.10.2 do sistema sem que nenhum gate notasse. Agora so' configura com o
# 6.12, e grava no build onde estao as ferramentas desse Qt
# (kinein-qt-bins.txt): os gates leem dali o qmllint e o runner dos harnesses,
# em vez de procurar no PATH.
#
# Onde procurar: KINEIN_QT_PREFIX, ou o mais novo ~/Qt/6.12.*/gcc_64 (o lugar
# do instalador oficial). Sem nenhum dos dois, o CMake procura no sistema e
# reprova se o de la' nao for o 6.12.

set(KINEIN_QT_VERSION 6.12)
set(KINEIN_QT_PREFIX "" CACHE PATH
    "Prefixo do Qt ${KINEIN_QT_VERSION} do projeto (vazio: o mais novo ~/Qt/${KINEIN_QT_VERSION}.*/gcc_64)")

set(kinein_qt_prefix "${KINEIN_QT_PREFIX}")
if(NOT kinein_qt_prefix)
    file(GLOB kinein_qt_candidates LIST_DIRECTORIES true
        "$ENV{HOME}/Qt/${KINEIN_QT_VERSION}.*/gcc_64")
    if(kinein_qt_candidates)
        list(SORT kinein_qt_candidates COMPARE NATURAL ORDER DESCENDING)
        list(GET kinein_qt_candidates 0 kinein_qt_prefix)
    endif()
endif()
if(kinein_qt_prefix)
    list(PREPEND CMAKE_PREFIX_PATH "${kinein_qt_prefix}")
endif()

# Chamada logo depois do find_package(Qt6 ...) da UI.
function(kinein_require_project_qt)
    string(REGEX MATCH "^[0-9]+\\.[0-9]+" found_minor "${Qt6_VERSION}")
    if(NOT found_minor VERSION_EQUAL KINEIN_QT_VERSION)
        message(FATAL_ERROR
            "O Qt do projeto e' o ${KINEIN_QT_VERSION} (roadmaps/40 §5), e este build "
            "achou o ${Qt6_VERSION} em ${Qt6_DIR}. Instale o ${KINEIN_QT_VERSION} em "
            "~/Qt/${KINEIN_QT_VERSION}.x/gcc_64 (DocsPublic/build/14-ambiente-de-desenvolvimento.md) "
            "ou passe -DKINEIN_QT_PREFIX=<prefixo>, e reconfigure com "
            "`cmake --fresh --preset <preset>`.")
    endif()
    file(WRITE "${CMAKE_BINARY_DIR}/kinein-qt-bins.txt"
        "${QT6_INSTALL_PREFIX}/${QT6_INSTALL_BINS}\n")
    message(STATUS "Qt do projeto: ${Qt6_VERSION} em ${QT6_INSTALL_PREFIX}")
endfunction()
