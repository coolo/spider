#ifndef _PILE_H_
#define _PILE_H_ 1

#include "card.h"
#include <QList>

const int MAX_CARDS = 104;

class Pile
{
private:
    Pile(const Pile &);

public:
    Pile()
    {
        count = 0;
    }
    Pile(Pile *other)
    {
        count = other->count;
        memcpy(cards, other->cards, MAX_CARDS);
    }
    void addCard(const Card &c);
    QString toString() const;
    bool empty() const { return count == 0; }
    Card at(int index) const { return cards[index]; }
    size_t cardCount() const { return count; }
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

private:
    Card cards[MAX_CARDS];
    size_t count;
};

#endif
