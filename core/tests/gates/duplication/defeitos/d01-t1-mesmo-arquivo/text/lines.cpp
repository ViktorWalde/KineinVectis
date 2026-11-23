// Corpus do gate de duplicação (P20), defeito d01: o lines.cpp da base com uma cópia.
#include "corpus/text/lines.hpp"

#include <algorithm>
#include <cstddef>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace corpus::text {

std::size_t count_lines(std::string_view text) {
    if (text.empty()) {
        return 0;
    }
    const auto breaks = static_cast<std::size_t>(std::ranges::count(text, '\n'));
    return text.back() == '\n' ? breaks : breaks + 1;
}

std::vector<ByteOffset> line_starts(std::string_view text) {
    std::vector<ByteOffset> starts{};
    if (text.empty()) {
        return starts;
    }
    starts.push_back(ByteOffset{0});
    for (std::size_t index{0}; index < text.size(); ++index) {
        const bool last = index + 1 == text.size();
        if (text[index] == '\n' && !last) {
            starts.push_back(ByteOffset{index + 1});
        }
    }
    return starts;
}

std::string_view trim(std::string_view text) {
    constexpr std::string_view blanks{" \t\r\n"};
    const auto first = text.find_first_not_of(blanks);
    if (first == std::string_view::npos) {
        return {};
    }
    const auto last = text.find_last_not_of(blanks);
    return text.substr(first, last - first + 1);
}

std::vector<std::string> wrap_words(std::string_view text, std::size_t width) {
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

// Defeito T1: copia exata de line_starts no mesmo arquivo, com outro nome.
std::vector<ByteOffset> newline_offsets(std::string_view text) {  // reprova: line_starts
    std::vector<ByteOffset> starts{};
    if (text.empty()) {
        return starts;
    }
    starts.push_back(ByteOffset{0});
    for (std::size_t index{0}; index < text.size(); ++index) {
        const bool last = index + 1 == text.size();
        if (text[index] == '\n' && !last) {
            starts.push_back(ByteOffset{index + 1});
        }
    }
    return starts;
}

}  // namespace corpus::text
