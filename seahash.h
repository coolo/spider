#pragma once

#include <cstdint>

struct SeahashState
{
    uint64_t a, b, c, d, written;

public:
    SeahashState()
    {
        a = 0x16f11fe89b0d677cULL;
        b = 0xb480a793d8e6c86cULL;
        c = 0x6fe2e5aaf078ebc9ULL;
        d = 0x14f994a4c5259381ULL;
        written = 0;
    }
    void push(uint64_t x);
    uint64_t finish() const;
};

uint64_t sea_hash(const void *key, int len);

struct HashHasher
{
    inline std::size_t operator()(const uint64_t &k) const
    {
        return std::size_t(k);
    }
};
