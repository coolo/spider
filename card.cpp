#include "card.h"
#include <QDebug>

QString Card::toString() const
{
    QString ret;
    switch (rank)
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
    default:
        if (rank < 2 || rank > 9)
            exit(1);
        ret += ('0' + rank);
        break;
    }
    switch (suit)
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
        qDebug() << "Invalid suit " << suit;
        exit(1);
    }
    if (!faceup)
        ret = "[" + ret + "]";
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
    qDebug() << "No map for " << c;
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
    qDebug() << "No map for " << c;
    exit(1);
    return Ace;
}
