#include "pile.h"
#include <QDebug>

Pile *Pile::copyFrom(Pile *from, int index)
{
    Pile *newone = new Pile(prefix);
    newone->cards = cards;
    for (int i = index; i < from->cardCount(); i++)
    {
        newone->cards.append(from->at(i));
    }
    newone->calculateChaos();
    return newone;
}

QString Pile::toString() const
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
    newone->calculateChaos();
    return newone;
}

void Pile::calculateChaos()
{
    m_chaos = 0;
    Card prev_card;
    for (Card c : cards)
    {
        if (!c.faceup)
        {
            m_chaos += 50;
        }
        else if (prev_card.rank != None)
        {
            if (prev_card.suit != c.suit)
                m_chaos += 5;
            if (prev_card.rank <= c.rank)
                m_chaos += 3 * (c.rank - prev_card.rank + 1);
            else if (c.rank == prev_card.rank - 1)
                m_chaos += 1;
            else
                m_chaos += 2;
        }
        else // first card face up
        {
            // first card king is least chaos
            m_chaos += (13 - c.rank);
        }
        prev_card = c;
    }
}