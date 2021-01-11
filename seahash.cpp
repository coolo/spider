#include <stdint.h>
#include <string.h>
#include <assert.h>
#include "seahash.h"
#include <stdio.h>

static inline uint64_t
diffuse(uint64_t val)
{
  uint64_t a, b;
  val *= 0x6eed0e9da4d94a4fULL;
  a = val >> 32;
  b = val >> 60;
  val ^= a >> b;
  val *= 0x6eed0e9da4d94a4fULL;
  return val;
}

void SeahashState::push(uint64_t x)
{
  // Mix `x` into `a`.

  uint64_t a2 = diffuse(a ^ x);

  //  Rotate around.
  //  _______________________
  // |                       v
  // a <---- b <---- c <---- d
  a = b;
  b = c;
  c = d;
  d = a2;

  // Increase the written bytes counter.
  written += 8;
}

uint64_t SeahashState::finish() const
{

  // XOR the states together. Even though XOR is commutative, it doesn't matter, because the
  // state vector's initial components are mutually distinct, and thus swapping even and odd
  // chunks will affect the result, because it is sensitive to the initial condition.
  uint64_t r1 = a ^ b;
  uint64_t r2 = c ^ d;
  r1 ^= r2;

  // XOR the number of written bytes in order to make the excessive bytes zero-sensitive
  // (without this, two excessive zeros would be equivalent to three excessive zeros). This
  // is know as length padding.
  r1 ^= written;

  // We diffuse to make the excessive bytes discrete (i.e. small changes shouldn't give small
  // changes in the output).
  return diffuse(r1);
}

uint64_t sea_hash(const void *key, int len, uint32_t seed)
{
  uint64_t a, b, c, d;
  uint64_t s = seed;
  uint64_t *p;
  unsigned char pad[8] = {0};

  a = 0x16f11fe89b0d677cULL ^ s;
  b = 0xb480a793d8e6c86cULL;
  c = 0x6fe2e5aaf078ebc9ULL;
  d = 0x14f994a4c5259381ULL;

  p = (uint64_t *)key;
  while (len >= 32)
  {
    a ^= *p++;
    b ^= *p++;
    c ^= *p++;
    d ^= *p++;
    a = diffuse(a);
    b = diffuse(b);
    c = diffuse(c);
    d = diffuse(d);
    len -= 32;
  }

  switch (len)
  {
  case 25 ... 31:
    a ^= *p++;
    b ^= *p++;
    c ^= *p++;
    memcpy(pad, p, len - 24);
    d ^= *(uint64_t *)pad;
    a = diffuse(a);
    b = diffuse(b);
    c = diffuse(c);
    d = diffuse(d);
    break;
  case 24:
    a ^= *p++;
    b ^= *p++;
    c ^= *p++;
    a = diffuse(a);
    b = diffuse(b);
    c = diffuse(c);
    break;
  case 17 ... 23:
    a ^= *p++;
    b ^= *p++;
    memcpy(pad, p, len - 16);
    c ^= *(uint64_t *)pad;
    a = diffuse(a);
    b = diffuse(b);
    c = diffuse(c);
    break;
  case 16:
    a ^= *p++;
    b ^= *p++;
    a = diffuse(a);
    b = diffuse(b);
    break;
  case 9 ... 15:
    a ^= *p++;
    memcpy(pad, p, len - 8);
    b ^= *(uint64_t *)pad;
    a = diffuse(a);
    b = diffuse(b);
    break;
  case 8:
    a ^= *p++;
    a = diffuse(a);
    break;
  case 1 ... 7:
    memcpy(pad, p, len);
    a ^= *(uint64_t *)pad;
    a = diffuse(a);
    break;
  case 0:
    break;
  default:
    assert(0);
  }

  a ^= b;
  c ^= d;
  a ^= c;
  a ^= (uint64_t)len;
  return diffuse(a);
}
