#include "pile.h"

Pile *Pile::copyFrom(Pile *from, int index)
{
    Pile *newone = new Pile(prefix);
    newone->cards = cards;
    for (int i = index; i < from->cardCount(); i++)
    {
        newone->cards.append(from->at(i));
    }
    return newone;
}

QString Pile::toString()
{
    QString ret = prefix;
    for (Card c : cards)
    {
        ret += " " + c.toString();
    }
    return ret;
}

Pile *Pile::remove(int index)
{
    Pile *newone = new Pile(prefix);
    newone->cards = cards;
    while (newone->cards.size() > index)
        newone->cards.removeLast();
    if (index > 0)
    {
        newone->cards[index - 1].faceup = true;
    }
    return newone;
}

bool Pile::addCard(QString token)
{
    Card newone;
    newone.faceup = !token.startsWith('|');
    if (!newone.faceup)
    {
        token.remove(0, 1);
    }
    newone.rank = newone.char2rank(token[0].toLatin1());
    newone.suit = newone.char2suit(token[1].toLatin1());
    cards.append(newone);
    return true;
}

Pile *Pile::newWithCard(const Card &c)
{
    Pile *newone = new Pile(prefix);
    newone->cards = cards;
    newone->cards.append(c);
    return newone;
}
