#ifndef _PILE_H_
#define _PILE_H_ 1

#include "card.h"
#include <QList>
#include "seahash.h"

const int MAX_CARDS = 104;

class Pile
{
public:
    Pile()
    {
        memset(cards, 0, MAX_CARDS + 1);
    }
    void addCard(const Card &c);
    std::string toString() const;
    bool empty() const { return count == 0; }
    const Card at(int index) const { return Card(cards[index]); }
    void setAt(int index, const Card &c) { cards[index] = c.raw_value(); }
    inline size_t cardCount() const { return count; }
    void remove(int index);
    void copyFrom(const Pile &from, int index);
    void replaceAt(int index, const Card &c);
    int chaos() const;
    void assignLeftCards(QList<Card> &list);
    void clear();
    void clone(const Pile &rhs);
    const unsigned char *cardsPtr() const { return (const unsigned char *)cards; };
    int sequenceOf(Suit suit) const;
    int playableCards() const;
    void updateHash(SeahashState &state) const;

private:
    uchar cards[MAX_CARDS];
    size_t count;
};

#endif
