#ifndef _PILE_H_
#define _PILE_H_ 1

#include "card.h"
#include "seahash.h"
#include <vector>

const int MAX_CARDS = 104;

class Pile;
class Pile
{
public:
    Pile()
    {
        count = 0;
    }
    const Pile *addCard(const Card &c) const;
    std::string toString() const;
    bool empty() const { return count == 0; }
    const Card at(int index) const { return Card(cards[index]); }

    inline size_t cardCount() const { return count; }
    const Pile *remove(int index) const;
    const Pile *copyFrom(const Pile *from, int index) const;
    const Pile *replaceAt(int index, const Card &c) const;
    int chaos() const { return m_chaos; }
    void calculateChaos();
    const Pile *assignLeftCards(std::vector<Card> &list) const;
    int sequenceOf(Suit suit) const { return m_seqs[suit]; }
    int playableCards() const;
    uint64_t hash() const { return m_hash; }
    static const Pile *createEmpty();

private:
    int m_chaos;
    void setAt(int index, const Card &c) { cards[index] = c.raw_value(); }
    unsigned char cards[MAX_CARDS];
    size_t count;
    uint64_t m_hash;
    static const Pile *query_or_insert(const unsigned char *cards, size_t count);
    int m_seqs[4];
    int sequenceOf_(Suit suit) const;
};

#endif
