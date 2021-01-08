#ifndef _DECK_H_
#define _DECK_H_

#include "pile.h"
#include "move.h"
#include <QList>
#include <QString>

const int MAX_MOVES = 230;

class Deck
{
public:
    Deck()
    {
        moves_index = 0;
    }
    Deck(const Deck &other);
    void getMoves(QVector<Move> &moves) const;
    QVector<Move> getWinMoves() const;
    QString toString() const;
    QString explainMove(Move m);
    void applyMove(const Move &m, Deck &newdeck, bool stop = false);
    uint64_t id() const;
    int leftTalons() const;
    int chaos() const;
    void assignLeftCards(QList<Card> &list);
    int shortestPath(int cap, bool debug);
    void addCard(int index, const Card &c);
    bool isWon() const;
    int playableCards() const;
    int inOff() const;
    int freePlays() const;

private:
    Pile play[10];
    Pile talon[5];
    Pile off;
    Move moves[MAX_MOVES];
    int moves_index;
};

#endif
