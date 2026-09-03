// Paleta e helpers de padrao compartilhados pelas partes do EditorHighlighter.
//
// POR QUE ESTE HEADER EXISTE (2026-09-03, etapa 16 do roadmaps/34). O
// `editor_highlighter.cpp` era o MAIOR arquivo do repositorio (910/500) e
// misturava cinco responsabilidades. Ao corta-lo por responsabilidade — mesmo
// precedente do `core_client_*.cpp`, uma classe em varios .cpp — o namespace
// anonimo com a paleta ficou sendo usado por mais de uma parte. Duplicar as
// cores em cada arquivo seria a forma silenciosa de elas divergirem: o realce
// de um token e o de uma marcacao deixariam de vir do MESMO amarelo sem que
// nada reclamasse. Um dono so', incluido por quem precisa.
#pragma once

#include <QColor>
#include <QFont>
#include <QRegularExpression>
#include <QStringList>
#include <QTextCharFormat>

namespace kinein::highlight {

// Paleta de docs/05-design-system.md, em QRgb para inicializacao constexpr.
inline constexpr QRgb kKeywordRgb = 0xffffbb00; // accent
inline constexpr QRgb kStringRgb = 0xff7fbf7f;  // success soft
inline constexpr QRgb kCommentRgb = 0xff8f8a7c; // text muted
inline constexpr QRgb kNumberRgb = 0xff7aa2d8;  // info soft
inline constexpr QRgb kMetaRgb = 0xffd16d6d;    // error soft (macros, atributos, secoes)

// Find/Replace (D1b): fundo de TODAS as ocorrencias e da ocorrencia ATUAL.
// Ambar da paleta em duas intensidades — a atual "acende" sob o cursor.
inline constexpr QRgb kSearchMatchRgb = 0xff4a3a12;   // accent bem rebaixado
inline constexpr QRgb kSearchCurrentRgb = 0xff8a6a1a; // accent dim, ocorrencia atual

// Cores semanticas (LSP): distinguir variaveis, funcoes, tipos e campos.
inline constexpr QRgb kTypeRgb = 0xff5fb3ac;     // tipos/classes/enums/namespaces
inline constexpr QRgb kFunctionRgb = 0xffd8a657; // funcoes/metodos
inline constexpr QRgb kVariableRgb = 0xffcdd6e4; // variaveis
inline constexpr QRgb kPropertyRgb = 0xffb48ead; // campos/propriedades/enum members

inline QTextCharFormat colorFormat(QRgb rgb, bool bold = false, bool italic = false)
{
    QTextCharFormat format;
    format.setForeground(QColor::fromRgb(rgb));
    if (bold) {
        format.setFontWeight(QFont::DemiBold);
    }
    format.setFontItalic(italic);
    return format;
}

inline QRegularExpression keywordPattern(const QStringList& keywords)
{
    return QRegularExpression(QStringLiteral("\\b(?:%1)\\b").arg(keywords.join(u'|')));
}

inline const QRegularExpression& numberPattern()
{
    static const QRegularExpression pattern(QStringLiteral(
        "\\b(?:0[xXbBoO][0-9a-fA-F_]+|\\d[\\d_]*(?:\\.\\d+)?(?:[eE][+-]?\\d+)?)\\b"));
    return pattern;
}

inline const QRegularExpression& doubleQuoteString()
{
    static const QRegularExpression pattern(QStringLiteral("\"(?:\\\\.|[^\"\\\\])*\""));
    return pattern;
}

inline const QRegularExpression& singleQuoteString()
{
    static const QRegularExpression pattern(QStringLiteral("'(?:\\\\.|[^'\\\\])*'"));
    return pattern;
}

} // namespace kinein::highlight
