#include "pile.h"
#include "SpookyV2.h"
#include <QDebug>
#include <QMap>

void Pile::copyFrom(const Pile &from, int index)
{
    for (int i = index; i < from.cardCount(); i++)
    {
        cards[count++] = from.at(i);
    }
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

void Pile::remove(int index)
{
    while (count > index)
    {
        cards[count - 1] = Card();
        count--;
    }
    if (index > 0)
    {
        cards[index - 1].set_faceup(true);
    }
}

void Pile::addCard(const Card &c)
{
    cards[count++] = c;
}

int Pile::chaos() const
{
    int chaos = 0;
    Card prev_card;
    int index = count;
    while (--index >= 0)
    {
        Card c = cards[index];
        if (!c.is_faceup())
        {
            chaos += 2;
        }
        else if (prev_card.rank() != None)
        {
            if (c.suit() == prev_card.suit() && c.rank() == prev_card.rank() + 1)
                chaos += 0;
            else
            {
                if (c.rank() < prev_card.rank())
                    chaos += 2;
                else
                    chaos += 1;
            }
        }
        else
        {
            chaos += 1;
        }
        prev_card = c;
    }
    //qDebug() << chaos << toString();
    return chaos;
}

void Pile::clear()
{
    for (int index = 0; index < count; index++)
    {
        cards[index] = Card();
    }
    count = 0;
}

void Pile::assignLeftCards(QList<Card> &list)
{
    for (int index = 0; index < count; index++)
    {
        if (cards[index].is_unknown())
        {
            Card c = list.takeFirst();
            c.set_faceup(cards[index].is_faceup());
            cards[index] = c;
        }
    }
}

void Pile::replaceAt(int index, const Card &c)
{
    cards[index] = c;
}

void Pile::clone(const Pile &rhs)
{
    count = rhs.count;
    memcpy(cards, rhs.cards, MAX_CARDS);
}

int Pile::sequenceOf(Suit suit) const
{
    int index = cardCount();
    if (index == 0)
    {
        return index;
    }
    index--;
    Card top_card = at(index);
    if (top_card.suit() != suit)
    {
        return 0;
    }
    while (index > 0 && top_card.inSequenceTo(at(index - 1)))
    {
        index--;
        top_card = at(index);
    }
    return cardCount() - index;
}

int Pile::playableCards() const
{
    if (count < 2)
    {
        return count;
    }
    Card top_card = at(count - 1);
    return sequenceOf(top_card.suit());
}