#ifndef _DECK_H_
#define _DECK_H_

#include "pile.h"
#include "move.h"
#include <QList>
#include <QString>

class Deck
{
public:
    Deck() { m_chaos = -17; }
    void addPile(Card *cards, size_t count);
    QList<Pile *> piles;
    QList<Move> getMoves();
    QString toString() const;
    QString explainMove(Move m);
    Deck *applyMove(Move m);
    uint64_t id();
    int chaos() const { return m_chaos; }
    void calculateChaos();

private:
    int m_chaos;
};

#endif