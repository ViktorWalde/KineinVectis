// Corpus of gate.duplication: the build module of the base tree.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include "corpus/build/diagnostics.hpp"

#include "corpus/text/lines.hpp"

#include <array>
#include <charconv>
#include <cstddef>
#include <expected>
#include <optional>
#include <string>
#include <string_view>
#include <system_error>

namespace corpus::build {

std::optional<Fields> split_fields(std::string_view text) {
    std::array<std::string_view, 4> heads{};
    std::size_t start{0};
    for (auto& head : heads) {
        const auto colon = text.find(':', start);
        if (colon == std::string_view::npos) {
            return std::nullopt;
        }
        head = text.substr(start, colon - start);
        start = colon + 1;
    }
    return Fields{
        .file = heads[0],
        .line = heads[1],
        .column = heads[2],
        .severity = text::trim(heads[3]),
        .message = text::trim(text.substr(start)),
    };
}

std::optional<std::size_t> parse_number(std::string_view digits) {
    std::size_t value{};
    const char* const end = digits.data() + digits.size();
    const auto [rest, error] = std::from_chars(digits.data(), end, value);
    if (error != std::errc{} || rest != end) {
        return std::nullopt;
    }
    return value;
}

std::optional<Severity> parse_severity(std::string_view word) {
    if (word == "note") {
        return Severity::note;
    }
    if (word == "warning") {
        return Severity::warning;
    }
    if (word == "error") {
        return Severity::error;
    }
    return std::nullopt;
}

std::string_view severity_name(Severity severity) {
    switch (severity) {
    case Severity::note:
        return "note";
    case Severity::warning:
        return "warning";
    case Severity::error:
        return "error";
    }
    return "unknown";
}

std::expected<Diagnostic, ParseError> parse_diagnostic(std::string_view text) {
    const auto fields = split_fields(text);
    if (!fields) {
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
    return Diagnostic{
        .file = std::string{fields->file},
        .line = *line,
        .column = *column,
        .severity = *severity,
        .message = std::string{fields->message},
    };
}

} // namespace corpus::build
