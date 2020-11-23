#include "pile.h"
#include <QDebug>
#include "SpookyV2.h"
#include <QMap>

QMap<uint64_t, Pile *> Pile::seen;

Pile *Pile::createPile(Card *newcards, size_t newcount)
{
    char bytes[104];
    int index = 0;
    for (int i = 0; i < newcount; i++)
    {
        bytes[index++] = newcards[i].asByte();
    }
    uint64_t id = SpookyHash::Hash64(bytes, index, 1);
    QMap<uint64_t, Pile *>::iterator seen_one = seen.find(id);
    if (seen_one != seen.end())
    {
        return *seen_one;
    }
    Pile *newone = new Pile();
    memcpy(newone->cards, newcards, sizeof(Card) * newcount);
    newone->count = newcount;
    newone->m_id = id;
    newone->calculateChaos();
    seen.insert(newone->m_id, newone);
    return newone;
}

Pile *Pile::copyFrom(Pile *from, int index)
{
    static Card newcards[104];
    memcpy(newcards, cards, sizeof(Card) * 104);
    int newcount = count;
    for (int i = index; i < from->cardCount(); i++)
    {
        newcards[newcount++] = from->at(i);
    }
    return createPile(newcards, newcount);
}

QString Pile::toString() const
{
    QString ret;
    for (int i = 0; i < count; i++)
    {
        ret += " " + cards[i].toString();
    }
    return ret;
}

Pile *Pile::remove(int index)
{
    static Card newcards[104];
    memcpy(newcards, cards, sizeof(Card) * count);
    size_t newcount = count;
    while (newcount > index)
        newcount--;
    if (index > 0)
    {
        newcards[index - 1].faceup = true;
    }
    return createPile(newcards, newcount);
}

Pile *Pile::newWithCard(const Card &c)
{
    static Card newcards[104];
    size_t newcount = count;
    memcpy(newcards, cards, sizeof(Card) * count);
    newcards[newcount++] = c;
    return createPile(newcards, newcount);
}

void Pile::calculateChaos()
{
    m_chaos = 0;
    Card prev_card;
    for (int i = 0; i < count; i++)
    {
        Card c = cards[i];
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
                m_chaos += 3;
        }
        else // first card face up
        {
            // first card king is least chaos
            m_chaos += (13 - c.rank) + 1;
        }
        prev_card = c;
    }
    //qDebug() << seen.count() << QString("%1").arg(m_id, 16, 16, QLatin1Char('0')) << m_chaos << toString();
}