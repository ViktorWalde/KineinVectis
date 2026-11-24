// Corpus of gate.duplication: defect d08, T3, wrap_words without two statements (the stop at the
// end of the text and the last flush).
//
// Documented in: docs/engineering/gates-and-effectiveness.md
#include <cstddef>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace corpus::text {

std::vector<std::string> wrap_simple(std::string_view text, std::size_t width) { // reject: wrap_words
    std::vector<std::string> lines{};
    std::string current{};
    std::size_t position{0};
    while (position < text.size()) {
        const auto start = text.find_first_not_of(' ', position);
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
    return lines;
}

} // namespace corpus::text
