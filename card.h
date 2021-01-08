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
    // 4 bits rank
    // 2 bits suit
    // 1 bit faceup
    // 1 bit unknown
    uchar value;

    Card()
    {
        value = 0;
    }

    inline bool is_faceup() const
    {
        return (value & (1 << 6)) > 0;
    }

    void set_faceup(bool face)
    {
        if (face)
        {
            value = value | (1 << 6);
        }
        else
        {
            value = value & !(1 << 6);
        }
    }

    inline bool is_unknown() const
    {
        return (value & (1 << 7)) > 0;
    }

    void set_unknown(bool unknown)
    {
        if (unknown)
        {
            value = value | (1 << 7);
        }
        else
        {
            value = value & ~(1 << 7);
        }
    }

    inline Rank rank() const
    {
        return Rank(value & 15);
    }

    void set_rank(Rank rank)
    {
        value = (value & ~15) + rank;
    }

    inline Suit suit() const
    {
        return Suit((value >> 4) & 3);
    }

    void set_suit(Suit suit)
    {
        Rank _rank = rank();
        value = value >> 4;
        value = (value & ~3) + suit;
        value = (value << 4) + _rank;
    }

    bool inSequenceTo(const Card &other) const;
    Card(QString token);
    QString toString() const;
    Suit char2suit(char c);
    Rank char2rank(char c);
    unsigned char raw_value() { return value; }
    bool operator==(const Card &rhs) const;
};

inline QDebug operator<<(QDebug debug, const Card &c)
{
    QDebugStateSaver saver(debug);
    debug.nospace() << c.toString();

    return debug;
}

inline bool Card::inSequenceTo(const Card &other) const
{
    return other.is_faceup() && other.suit() == suit() && other.rank() == rank() + 1;
}

#endif
