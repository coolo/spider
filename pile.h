#ifndef _PILE_H_
#define _PILE_H_ 1

#include "card.h"
#include <QList>

class Pile
{
public:
    Pile(QString _prefix) { prefix = _prefix; }
    bool addCard(QString token);
    Pile *newWithCard(const Card &c);
    QString toString() const;
    QString name() const { return prefix; }
    bool empty() const { return cards.empty(); }
    Card at(int index) const { return cards[index]; }
    size_t cardCount() const { return cards.count(); }
    Pile *remove(int index);
    Pile *copyFrom(Pile *from, int index);
    int chaos() const { return m_chaos; }

private:
    void calculateChaos();
    int m_chaos;
    QString prefix;
    QList<Card> cards;
};

#endif