// Corpus do gate de duplicação (P20), forma correta c06: construtores e
// operações especiais com a mesma forma, em classes diferentes.
#include <cstddef>
#include <string>
#include <vector>

namespace corpus::text {

class LineBuffer {
public:
    explicit LineBuffer(std::size_t capacity) : lines_(capacity), capacity_{capacity} {}
    LineBuffer(const LineBuffer&) = default;
    LineBuffer(LineBuffer&&) noexcept = default;
    LineBuffer& operator=(const LineBuffer&) = default;
    LineBuffer& operator=(LineBuffer&&) noexcept = default;
    ~LineBuffer() = default;

    [[nodiscard]] std::size_t capacity() const noexcept { return capacity_; }

private:
    std::vector<std::string> lines_{};
    std::size_t capacity_{0};
};

class ByteBuffer {
public:
    explicit ByteBuffer(std::size_t capacity) : bytes_(capacity), capacity_{capacity} {}
    ByteBuffer(const ByteBuffer&) = default;
    ByteBuffer(ByteBuffer&&) noexcept = default;
    ByteBuffer& operator=(const ByteBuffer&) = default;
    ByteBuffer& operator=(ByteBuffer&&) noexcept = default;
    ~ByteBuffer() = default;

    [[nodiscard]] std::size_t capacity() const noexcept { return capacity_; }

private:
    std::vector<char> bytes_{};
    std::size_t capacity_{0};
};

}  // namespace corpus::text
