#ifndef _PILE_H_
#define _PILE_H_ 1

#include "card.h"
#include <QList>

const int MAX_CARDS = 104;

class Pile
{
public:
    Pile()
    {
        m_chaos = 0;
        count = 0;
    }
    Pile(Pile *other)
    {
        count = other->count;
        m_chaos = other->m_chaos;
        memcpy(cards, other->cards, MAX_CARDS);
    }
    void addCard(const Card &c);
    QString toString() const;
    bool empty() const { return count == 0; }
    Card at(int index) const { return cards[index]; }
    size_t cardCount() const { return count; }
    void remove(int index);
    void copyFrom(Pile *from, int index);
    void replaceAt(int index, const Card &c);
    int chaos() const { return m_chaos; }
    void assignLeftCards(QList<Card> &list);
    void clear();
    const unsigned char *cardsPtr() const { return (const unsigned char *)cards; };

private:
    void calculateChaos();
    int m_chaos;
    Card cards[MAX_CARDS];
    size_t count;
};

#endif
