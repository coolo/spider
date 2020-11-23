#include "deck.h"
#include "move.h"
#include "pile.h"
#include "card.h"
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

        int count = piles[from]->cardCount() - 1;
        Suit top_suit = piles[from]->at(count).suit;
        int top_rank = int(piles[from]->at(count).rank) - 1;

        while (count >= 0)
        {
            Card current = piles[from]->at(count);
            if (!current.faceup)
                break;
            if (current.suit != top_suit)
                break;
            if (top_rank + 1 != current.rank)
                break;
            top_rank = piles[from]->at(count).rank;

            for (int to = 0; to < 10; to++)
            {
                if (to == from)
                    continue;
                //qDebug() << "trying to move " << (piles[from]->cardCount() - count) << " from " << from << " to " << to;
                size_t to_count = piles[to]->cardCount() - 1;
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
                ret.last().index = count;
            }
            count--;
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
        newone->piles[m.from] = new Pile(newone->piles[m.from]->name());
        return newone;
    }
    newone->piles[m.to] = newone->piles[m.to]->copyFrom(newone->piles[m.from], m.index);
    newone->piles[m.from] = newone->piles[m.from]->remove(m.index);
    return newone;
}

QString Deck::toString() const
{
    QString ret;
    for (Pile *p : piles)
    {
        ret += p->toString();
        ret += QStringLiteral("\n");
    }
    return ret;
}

Pile *Deck::addPile(QString token)
{
    Pile *p = new Pile(token);
    piles.append(p);
    return p;
}