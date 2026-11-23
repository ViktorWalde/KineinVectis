// Corpus of gate.duplication: defect d07, T3, parse_diagnostic with two extra validations.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include "corpus/build/diagnostics.hpp"

#include <expected>
#include <string>
#include <string_view>

namespace corpus::build {

std::expected<Diagnostic, ParseError> parse_strict_diagnostic(std::string_view text) { // reject: parse_diagnostic
    const auto fields = split_fields(text);
    if (!fields) {
        return std::unexpected(ParseError::missing_location);
    }
    if (fields->file.empty()) {
        return std::unexpected(ParseError::missing_location);
    }
    const auto line = parse_number(fields->line);
    if (!line) {
        return std::unexpected(ParseError::bad_line_number);
    }
    const auto column = parse_number(fields->column);
    if (!column) {
        return std::unexpected(ParseError::bad_column);
    }
    const auto severity = parse_severity(fields->severity);
    if (!severity) {
        return std::unexpected(ParseError::unknown_severity);
    }
    if (*line == 0 || *column == 0) {
        return std::unexpected(ParseError::bad_line_number);
    }
    return Diagnostic{
        .file = std::string{fields->file},
        .line = *line,
        .column = *column,
        .severity = *severity,
        .message = std::string{fields->message},
    };
}

} // namespace corpus::build
