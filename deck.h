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
    }
    Deck(const Deck &other);
    QList<Move> getMoves();
    QString toString() const;
    QString explainMove(Move m);
    Deck *applyMove(Move m, bool stop = false);
    uint64_t id() const;
    int leftTalons() const;
    int chaos() const;
    QList<Move> order;
    void assignLeftCards(QList<Card> &list);
    int shortestPath(int cap, bool debug);
    int free_talons() const;
    void addCard(int index, const Card &c);
    bool operator<(const Deck &rhs) const;
    bool is_won() const;

private:
    Pile play[10];
    Pile talon[5];
    Pile off;
};

#endif
