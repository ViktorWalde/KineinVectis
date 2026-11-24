// Corpus of gate.duplication: defect d09, T3, wrap_words with another comparison and one
// replaced statement.
//
// Documented in: docs/engineering/gates-and-effectiveness.md
#include <cstddef>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace corpus::protocol {

std::vector<std::string> layout_words(std::string_view text, std::size_t width) { // reject: wrap_words
    std::vector<std::string> lines{};
    std::string current{};
    std::size_t position{0};
    while (position < text.size()) {
        const auto start = text.find_first_not_of(' ', position);
        if (start == std::string_view::npos) {
            break;
        }
        auto end = text.find(' ', start);
        if (end == std::string_view::npos) {
            end = text.size();
        }
        const auto word = text.substr(start, end - start);
        if (!current.empty() && current.size() + 1 + word.size() >= width) {
            lines.push_back(std::move(current));
            current.clear();
        }
        if (!current.empty()) {
            current.append(" ");
        }
        current.append(word);
        position = end;
    }
    if (!current.empty()) {
        lines.push_back(std::move(current));
    }
    return lines;
}

} // namespace corpus::protocol
