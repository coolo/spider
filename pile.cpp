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
    Pile *newone = new Pile(prefix);
    newone->cards = cards;
    for (int i = index; i < from->cardCount(); i++)
    {
        newone->cards.append(from->at(i));
    }
    return checkIfNew(newone);
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
    cards.append(newone);
    calculateChaos();
    calculateId();
    return true;
}

Pile *Pile::newWithCard(const Card &c)
{
    Pile *newone = new Pile(prefix);
    newone->cards = cards;
    newone->cards.append(c);
    return checkIfNew(newone);
}

void Pile::calculateId()
{
    QByteArray representation = toString().toLocal8Bit();
    m_id = SpookyHash::Hash64(representation.data(), representation.size(), 1);
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