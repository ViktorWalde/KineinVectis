// Corpus of gate.duplication: correct form c14, functions with the same name in different modules,
// with different logic.
//
// Documented in: docs/engineering/gates-and-effectiveness.md
#include <cstddef>
#include <string_view>
#include <vector>

namespace corpus::build {

std::vector<std::string_view> parse(std::string_view command_line) {
    std::vector<std::string_view> arguments{};
    std::size_t position{0};
    while (position < command_line.size()) {
        const auto start = command_line.find_first_not_of(' ', position);
        if (start == std::string_view::npos) {
            break;
        }
        const bool quoted = command_line[start] == '"';
        const auto stop = quoted ? command_line.find('"', start + 1) : command_line.find(' ', start);
        const auto end = stop == std::string_view::npos ? command_line.size() : stop;
        arguments.push_back(command_line.substr(quoted ? start + 1 : start, end - (quoted ? start + 1 : start)));
        position = end + 1;
    }
    return arguments;
}

} // namespace corpus::build
