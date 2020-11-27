#ifndef _CARD_H_
#define _CARD_H_ 1

#include <QString>
#include <QDebug>

enum Suit
{
    Spades = 0,
    Hearts = 1,
    Clubs = 2,
    Diamonds = 3
};
enum Rank
{
    None = 0,
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
    bool unknown;
    Suit suit;
    Rank rank;

    Card()
    {
        faceup = false;
        rank = None;
        unknown = true;
    }
    Card(QString token);
    QString toString() const;
    Suit char2suit(char c);
    Rank char2rank(char c);
    // 4 bits for rank, 2 bits for suit, 1 bit for faceup
    unsigned char asByte() { return rank + (suit << 4) + (faceup << 6); }
    bool operator==(const Card &rhs) const;
};

inline QDebug operator<<(QDebug debug, const Card &c)
{
    QDebugStateSaver saver(debug);
    debug.nospace() << c.toString();

    return debug;
}
#endif
