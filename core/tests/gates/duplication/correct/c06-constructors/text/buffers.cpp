// Corpus of gate.duplication: correct form c06, constructors and special member functions with the
// same shape, in different classes.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
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

} // namespace corpus::text
