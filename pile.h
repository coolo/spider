#ifndef _PILE_H_
#define _PILE_H_ 1

#include "card.h"
#include <QList>

class Pile
{
private:
    Pile()
    {
        m_id = 0;
        m_chaos = 0;
        count = 0;
    }

public:
    Pile *newWithCard(const Card &c);
    QString toString() const;
    bool empty() const { return count == 0; }
    Card at(int index) const { return cards[index]; }
    size_t cardCount() const { return count; }
    Pile *remove(int index);
    Pile *copyFrom(Pile *from, int index);
    Pile *replaceAt(int index, const Card &c);
    int chaos() const { return m_chaos; }
    uint64_t id() const { return m_id; }
    static Pile *createPile(Card *cards, size_t count);
    Pile *assignLeftCards(QList<Card> &list);

private:
    void calculateChaos();
    int m_chaos;
    uint64_t m_id;
    Card cards[104];
    size_t count;
    static QMap<uint64_t, Pile *> seen;
};

#endif
