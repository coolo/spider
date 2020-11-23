#include "pile.h"
#include <QDebug>
#include "SpookyV2.h"
#include <QMap>

static QMap<uint64_t, Pile *> seen;

Pile *Pile::checkIfNew(Pile *newone)
{
    newone->calculateId();
    QMap<uint64_t, Pile *>::iterator seen_one = seen.find(newone->m_id);
    if (seen_one != seen.end())
    {
        delete newone;
        return *seen_one;
    }
    newone->calculateChaos();
    seen.insert(newone->m_id, newone);
    return newone;
}

Pile *Pile::copyFrom(Pile *from, int index)
{
    Pile *newone = new Pile();
    memcpy(newone->cards, cards, sizeof(Card) * 104);
    newone->count = count;
    for (int i = index; i < from->cardCount(); i++)
    {
        newone->cards[newone->count++] = from->at(i);
    }
    return checkIfNew(newone);
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
    Pile *newone = new Pile();
    memcpy(newone->cards, cards, sizeof(Card) * 104);
    newone->count = count;
    while (newone->count > index)
        newone->count--;
    if (index > 0)
    {
        newone->cards[index - 1].faceup = true;
    }
    return checkIfNew(newone);
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
    cards[count++] = newone;
    calculateChaos();
    calculateId();
    return true;
}

Pile *Pile::newWithCard(const Card &c)
{
    Pile *newone = new Pile();
    memcpy(newone->cards, cards, sizeof(Card) * 104);
    newone->count = count;
    newone->cards[newone->count++] = c;
    return checkIfNew(newone);
}

void Pile::calculateId()
{
    // we have max 104 cards (and all of them in one pile is rather strange)
    // each can be represented by a byte
    char bytes[104];
    int index = 0;
    for (int i = 0; i < count; i++)
    {
        bytes[index++] = cards[i].asByte();
    }
    m_id = SpookyHash::Hash64(bytes, index, 1);
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