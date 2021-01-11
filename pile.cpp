#include "pile.h"
#include <QDebug>
#include <QMap>

void Pile::copyFrom(const Pile &from, int index)
{
    for (int i = index; i < from.cardCount(); i++)
    {
        setAt(cardCount(), from.at(i));
        cards[0]++;
    }
}

std::string Pile::toString() const
{
    std::string ret;
    for (int i = 0; i < cards[0]; i++)
    {
        ret += " " + Card(cards[i + 1]).toString();
    }
    return ret;
}

void Pile::remove(int index)
{
    while (cards[0] > index)
    {
        cards[cards[0]] = 0;
        cards[0]--;
    }
    if (index > 0)
    {
        Card c = at(index - 1);
        c.set_faceup(true);
        setAt(index - 1, c);
    }
}

void Pile::addCard(const Card &c)
{
    setAt(cardCount(), c);
    cards[0]++;
}

int Pile::chaos() const
{
    int result = 0;
    Card lastcard;
    for (int i = 0; i < cardCount(); i++)
    {
        Card current = at(i);

        // first in stack
        if (lastcard.raw_value() == 0)
        {
            result++;
        }
        else
        {
            if (!current.inSequenceTo(lastcard))
            {
                result++;
            }
        }
        lastcard = current;
    }
    return result;
}

void Pile::clear()
{
    memset(cards, 0, MAX_CARDS + 1);
}

void Pile::assignLeftCards(QList<Card> &list)
{
    for (int index = 0; index < cardCount(); index++)
    {
        if (at(index).is_unknown())
        {
            Card c = list.takeFirst();
            c.set_faceup(at(index).is_faceup());
            cards[index + 1] = c.raw_value();
        }
    }
}

void Pile::replaceAt(int index, const Card &c)
{
    cards[index + 1] = c.raw_value();
}

void Pile::clone(const Pile &rhs)
{
    memcpy(cards, rhs.cards, MAX_CARDS + 1);
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
    if (cards[0] < 2)
    {
        return cards[0];
    }
    return sequenceOf(at(cards[0] - 1).suit());
}

void Pile::updateHash(SeahashState &state) const
{
    const uchar *ptr = cards;
    const uchar *max_offset = ptr + cardCount() + 1;

    do
    {
        state.push(*((uint64_t *)ptr));
        ptr += 8;
    } while (ptr <= max_offset);
}
