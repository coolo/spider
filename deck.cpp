#include "deck.h"
#include "move.h"
#include "pile.h"
#include "card.h"
#include "SpookyV2.h"
#include <QList>
#include <QDebug>

QList<Move> Deck::getMoves()
{
    QList<Move> ret;
    int from = 0;
    for (; from < 10; from++)
    {
        //qDebug() << "Play" << piles[from]->toString();
        if (piles[from]->empty())
            continue;

        int index = piles[from]->cardCount() - 1;
        Suit top_suit = piles[from]->at(index).suit;
        int top_rank = int(piles[from]->at(index).rank) - 1;

        while (index >= 0)
        {
            Card current = piles[from]->at(index);
            if (!current.faceup)
                break;
            if (current.suit != top_suit)
                break;
            if (top_rank + 1 != current.rank)
                break;
            top_rank = piles[from]->at(index).rank;

            if (piles[from]->cardCount() - index == 13)
            {
                ret.append(Move());
                ret.last().from = from;
                ret.last().to = 0;
                ret.last().off = true;
                ret.last().index = index;
                continue;
            }
            for (int to = 0; to < 10; to++)
            {
                if (to == from)
                    continue;
                //qDebug() << "trying to move " << (piles[from]->cardCount() - index) << " from " << from << " to " << to;
                int to_count = piles[to]->cardCount() - 1;
                if (to_count > 0)
                {
                    Card top_card = piles[to]->at(to_count);
                    if (top_card.rank != top_rank + 1)
                    {
                        //qDebug() << "no match";
                        continue;
                    }
                }
                ret.append(Move());
                ret.last().from = from;
                ret.last().to = to;
                ret.last().index = index;
            }
            index--;
        }
    }
    from = 10;
    for (; from < 15; from++)
    {
        if (!piles[from]->empty())
        {
            ret.append(Move());
            ret.last().from = from;
            ret.last().talon = true;
            break;
        }
    }
    return ret;
}

QString Deck::explainMove(Move m)
{
    if (m.talon)
    {
        return "Draw another talon";
    }
    if (m.off)
    {
        return QString("Move a sequence from %1 to the off").arg(m.from);
    }
    QString fromCard = piles[m.from]->at(m.index).toString();
    QString toCard = piles[m.to]->at(piles[m.to]->cardCount() - 1).toString();
    return QString("Move %1 cards from %2 to %3 - %4->%5").arg(piles[m.from]->cardCount() - m.index).arg(m.from).arg(m.to).arg(fromCard).arg(toCard);
}

Deck *Deck::applyMove(Move m)
{
    Deck *newone = new Deck;
    newone->piles = piles;
    if (m.talon)
    {
        for (int to = 0; to < 10; to++)
        {
            Card c = newone->piles[m.from]->at(to);
            c.faceup = true;
            newone->piles[9 - to] = newone->piles[9 - to]->newWithCard(c);
        }
        // empty pile
        newone->piles[m.from] = Pile::createPile(0,0);
    }
    else if (m.off)
    {
        Card c = newone->piles[m.from]->at(newone->piles[m.from]->cardCount() - 13);
        newone->piles[15] = newone->piles[15]->newWithCard(c);
        newone->piles[m.from] = newone->piles[m.from]->remove(m.index);
    }
    else
    {
        newone->piles[m.to] = newone->piles[m.to]->copyFrom(newone->piles[m.from], m.index);
        newone->piles[m.from] = newone->piles[m.from]->remove(m.index);
    }
    newone->calculateChaos();
    return newone;
}

QString Deck::toString() const
{
    QString ret;
    int counter = 0;
    for (Pile *p : piles)
    {
        if (counter < 10)
        {
            ret += QString("Play%1:").arg(counter);
        }
        else if (counter < 15)
        {
            ret += QString("Deck%1:").arg(counter - 10);
        }
        else
            ret += "Off:";

        ret += p->toString();
        ret += QStringLiteral("\n");
        counter++;
    }
    return ret;
}

void Deck::addPile(Card *cards, size_t count)
{
    piles.append(Pile::createPile(cards,count));
}

uint64_t Deck::id()
{
    uint64_t ids[16];
    int counter = 0;
    for (Pile *p : piles)
    {
        ids[counter++] = p->id();
    }
    return SpookyHash::Hash64(&ids, 16 * 8, 1);
}

void Deck::calculateChaos()
{
    m_chaos = 0;
    for (int i = 0; i < 10; i++)
    {
        m_chaos += piles[i]->chaos();
    }
    for (int i = 10; i < 15; i++)
    {
        if (!piles[i]->empty())
            m_chaos += 100;
    }
}