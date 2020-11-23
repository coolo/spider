#ifndef _CARD_H_
#define _CARD_H_ 1

#include <QString>

enum Suit
{
    Spades,
    Hearts,
    Clubs,
    Diamonds
};
enum Rank
{
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13
};

struct Card
{
    bool faceup;
    Suit suit;
    Rank rank;

    QString toString() const;
    Suit char2suit(char c);
    Rank char2rank(char c);
};

#endif