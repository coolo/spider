#ifndef _DECK_H_
#define _DECK_H_

#include "pile.h"
#include "move.h"
#include <QList>
#include <QString>

class Deck
{
public:
    Deck()
    {
        m_chaos = -17;
        m_moves = 0;
    }
    void addPile(Card *cards, size_t count);
    QList<Move> getMoves();
    QString toString() const;
    QString explainMove(Move m);
    Deck *applyMove(Move m);
    uint64_t id();
    int chaos() const { return m_chaos; }
    int moves() const { return m_moves; }
    int leftTalons() const { return m_talons; }
    void calculateChaos();
    QList<Move> order;

private:
    QList<Pile *> piles;

    int m_moves;
    int m_chaos;
    int m_talons;
};

#endif
