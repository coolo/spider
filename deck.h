#ifndef _DECK_H_
#define _DECK_H_

#include "pile.h"
#include "move.h"
#include <vector>

const int MAX_MOVES = 230;

class Deck
{
private:
    // Deck(const Deck &other);

public:
    Deck()
    {
        // leaving the pointers stale on purpose
        // for performance
    }

    void update(const Deck *);
    void getMoves(std::vector<Move> &moves) const;
    std::vector<Move> getWinMoves() const;
    std::string toString() const;
    std::string explainMove(Move m);
    void applyMove(const Move &m, Deck &newdeck, bool stop = false);
    uint64_t id() const;
    int leftTalons() const;
    int chaos() const;
    void assignLeftCards(std::vector<Card> &list);
    int shortestPath(int cap, bool debug);
    void addCard(int index, const Card &c);
    bool isWon() const;
    int playableCards() const;
    int inOff() const;
    int freePlays() const;
    void makeEmpty();

private:
    const Pile *play[10];
    const Pile *talon[5];
    const Pile *off;
    Move moves[MAX_MOVES];
    int moves_index;
};

#endif
