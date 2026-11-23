// Corpus of gate.duplication: the build module of the base tree.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#ifndef CORPUS_BUILD_DIAGNOSTICS_HPP
#define CORPUS_BUILD_DIAGNOSTICS_HPP

#include <cstddef>
#include <expected>
#include <optional>
#include <string>
#include <string_view>

namespace corpus::build {

enum class Severity { note, warning, error };
enum class ParseError { missing_location, bad_line_number, bad_column, unknown_severity };

struct Diagnostic {
    std::string file{};
    std::size_t line{};
    std::size_t column{};
    Severity severity{};
    std::string message{};
};

// The fields of "file:line:column: severity: message".
struct Fields {
    std::string_view file{};
    std::string_view line{};
    std::string_view column{};
    std::string_view severity{};
    std::string_view message{};
};

[[nodiscard]] std::optional<Fields> split_fields(std::string_view text);
[[nodiscard]] std::optional<std::size_t> parse_number(std::string_view digits);
[[nodiscard]] std::optional<Severity> parse_severity(std::string_view word);
[[nodiscard]] std::string_view severity_name(Severity severity);
[[nodiscard]] std::expected<Diagnostic, ParseError> parse_diagnostic(std::string_view text);

} // namespace corpus::build

#endif
