#ifndef _PILE_H_
#define _PILE_H_ 1

#include "card.h"
#include <QList>

class Pile
{
public:
    Pile() { m_id = 0; m_chaos = 0; count = 0; }
    bool addCard(QString token);
    Pile *newWithCard(const Card &c);
    QString toString() const;
    bool empty() const { return count == 0; }
    Card at(int index) const { return cards[index]; }
    size_t cardCount() const { return count; }
    Pile *remove(int index);
    Pile *copyFrom(Pile *from, int index);
    int chaos() const { return m_chaos; }
    uint64_t id() const { return m_id; }

private:
    void calculateId();
    void calculateChaos();
    int m_chaos;
    uint64_t m_id;
    Card cards[104];
    size_t count;
    static Pile *checkIfNew(Pile *newone);
};

#endif