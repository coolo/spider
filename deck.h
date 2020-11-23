#ifndef _DECK_H_
#define _DECK_H_

#include "pile.h"
#include "move.h"
#include <QList>
#include <QString>

class Deck
{
public:
    Pile *addPile(QString token);
    QList<Pile *> piles;
    QList<Move> getMoves();
    QString toString() const;
    QString explainMove(Move m);
    Deck *applyMove(Move m);
};

#endif