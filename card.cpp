#include "card.h"
#include <string>
#include <QDebug>

std::string Card::toString() const
{
    if (is_unknown() && is_faceup())
        return "XX";
    if (is_unknown())
        return "|XX";

    std::string ret;
    switch (rank())
    {
    case Ace:
        ret += 'A';
        break;
    case Jack:
        ret += 'J';
        break;
    case Queen:
        ret += 'Q';
        break;
    case King:
        ret += 'K';
        break;
    case Ten:
        ret += 'T';
        break;
    case None:
        ret += 'N';
        break;
    default:
        if (rank() < 2 || rank() > 9)
        {
            qFatal("rank is out of range");
            exit(1);
        }
        ret += ('0' + rank());
        break;
    }
    switch (suit())
    {
    case Spades:
        ret += "S";
        break;
    case Hearts:
        ret += "H";
        break;
    case Diamonds:
        ret += "D";
        break;
    case Clubs:
        ret += "C";
        break;
    default:
        qDebug() << "Invalid suit " << suit();
        exit(1);
    }
    if (!is_faceup())
        ret = "|" + ret;
    return ret;
}

Suit Card::char2suit(char c)
{
    switch (c)
    {
    case 'S':
        return Spades;
    case 'H':
        return Hearts;
    case 'D':
        return Diamonds;
    case 'C':
        return Clubs;
    }
    qDebug() << "No suit map for" << c;
    exit(1);
    return Spades;
}

Rank Card::char2rank(char c)
{
    switch (c)
    {
    case 'K':
        return King;
    case 'Q':
        return Queen;
    case 'A':
        return Ace;
    case 'T':
        return Ten;
    case 'J':
        return Jack;
    case '2':
        return Two;
    case '3':
        return Three;
    case '4':
        return Four;
    case '5':
        return Five;
    case '6':
        return Six;
    case '7':
        return Seven;
    case '8':
        return Eight;
    case '9':
        return Nine;
    }
    qDebug() << "No rank for" << c;
    exit(1);
    return Ace;
}

Card::Card(const std::string &token_)
{
    std::string token = token_;
    value = 0;
    set_faceup(token.find('|') != 0);
    if (!is_faceup())
    {
        token.erase(0, 1);
    }
    if (token == "XX")
    {
        set_rank(None);
        set_suit(Spades);
        set_unknown(true);
        return;
    }

    set_rank(char2rank(token[0]));
    set_suit(char2suit(token[1]));
    set_unknown(false);
}

// to remove known cards from partly unknown decks. We don't care for faceup and unknown
bool Card::operator==(const Card &rhs) const
{
    return suit() == rhs.suit() && rank() == rhs.rank();
}
