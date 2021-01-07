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
    Deck(const Deck &other);
    QList<Move> getMoves();
    QString toString() const;
    QString explainMove(Move m);
    Deck *applyMove(Move m, bool stop = false);
    uint64_t id();
    int chaos() const { return m_chaos; }
    int moves() const { return m_moves; }
    int leftTalons() const { return m_talons; }
    void calculateChaos();
    QList<Move> order;
    void assignLeftCards(QList<Card> &list);
    int shortestPath(int cap, bool debug);
    int free_talons() const;
    void addCard(int index, const Card &c);

private:
    Pile play[10];
    Pile talon[5];
    Pile off;

    int m_moves;
    int m_chaos;
    int m_talons;
};

#endif
