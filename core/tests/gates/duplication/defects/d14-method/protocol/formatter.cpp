// Corpus of gate.duplication: defect d14, wrap_words copied as a static method of a class in
// another module.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include <cstddef>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace corpus::protocol {

class Formatter {
public:
    static std::vector<std::string> wrap(std::string_view text, std::size_t width) { // reject: wrap_words
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
            lines.push_back(std::move(current));
        }
        return lines;
    }
};

} // namespace corpus::protocol
