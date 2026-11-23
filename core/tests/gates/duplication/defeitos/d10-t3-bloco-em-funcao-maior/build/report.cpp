// Corpus do gate de duplicação (P20), defeito d10: o laço de wrap_words
// copiado dentro de uma função maior, que faz outras coisas.
#include "corpus/build/diagnostics.hpp"

#include <cstddef>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace corpus::build {

std::vector<std::string> format_report(const Diagnostic& diagnostic, std::size_t width) {
    std::vector<std::string> lines{};
    std::string header{diagnostic.file};
    header.append(":");
    header.append(std::to_string(diagnostic.line));
    header.append(": ");
    header.append(severity_name(diagnostic.severity));
    lines.push_back(std::move(header));
    const std::string_view text{diagnostic.message};
    std::string current{};
    std::size_t position{0};
    while (position < text.size()) {  // reprova: wrap_words
        const auto start = text.find_first_not_of(' ', position);
        if (start == std::string_view::npos) {
            break;
        }
        auto end = text.find(' ', start);
        if (end == std::string_view::npos) {
            end = text.size();
        }
        const auto word = text.substr(start, end - start);
        if (!current.empty() && current.size() + 1 + word.size() > width) {
            lines.push_back(std::move(current));
            current.clear();
        }
        if (!current.empty()) {
            current.push_back(' ');
        }
        current.append(word);
        position = end;
    }
    if (!current.empty()) {
        lines.push_back("    " + current);
    }
    lines.emplace_back();
    return lines;
}

}  // namespace corpus::build
