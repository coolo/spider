#include "pile.h"
#include <QDebug>
#include <QMap>
#include <unordered_map>

static std::unordered_map<uint64_t, Pile *> pilemap;

const Pile *Pile::query_or_insert(const unsigned char *cards, size_t count)
{
    uint64_t h = sea_hash(cards, count);
    auto search = pilemap.find(h);
    if (search != pilemap.end())
        return search->second;
    Pile *p = new Pile;
    memcpy(p->cards, cards, MAX_CARDS);
    p->count = count;
    p->calculateChaos();
    p->m_hash = h;
    p->m_seqs[Hearts] = p->sequenceOf_(Hearts);
    p->m_seqs[Spades] = p->sequenceOf_(Spades);
    p->m_seqs[Clubs] = p->sequenceOf_(Clubs);
    p->m_seqs[Diamonds] = p->sequenceOf_(Diamonds);
    pilemap.insert({h, p});
    return p;
}

const Pile *Pile::createEmpty()
{
    unsigned char newcards[MAX_CARDS];
    memset(newcards, 0, MAX_CARDS);
    return query_or_insert(newcards, 0);
}

const Pile *Pile::copyFrom(const Pile *from, int index) const
{
    unsigned char newcards[MAX_CARDS];
    memcpy(newcards, cards, count);
    size_t newcount = count;
    for (size_t i = index; i < from->count; i++)
    {
        newcards[newcount++] = from->cards[i];
    }
    memset(newcards + newcount, 0, MAX_CARDS - newcount);
    return query_or_insert(newcards, newcount);
}

std::string Pile::toString() const
{
    std::string ret;
    for (int i = 0; i < count; i++)
    {
        ret += " " + Card(cards[i]).toString();
    }
    return ret;
}

const Pile *Pile::remove(int index) const
{
    unsigned char newcards[MAX_CARDS];
    memcpy(newcards, cards, MAX_CARDS);
    size_t newcount = count;

    while (newcount > index)
    {
        newcards[newcount] = 0;
        newcount--;
    }
    if (index > 0)
    {
        Card c = at(index - 1);
        c.set_faceup(true);
        newcards[index - 1] = c.raw_value();
    }
    return query_or_insert(newcards, newcount);
}

const Pile *Pile::addCard(const Card &c) const
{
    unsigned char newcards[MAX_CARDS];
    memcpy(newcards, cards, MAX_CARDS);
    newcards[count] = c.raw_value();
    return query_or_insert(newcards, count + 1);
}

void Pile::calculateChaos()
{
    m_chaos = 0;
    Card lastcard;
    for (int i = 0; i < cardCount(); i++)
    {
        Card current = at(i);

        // first in stack
        if (lastcard.raw_value() == 0)
        {
            m_chaos++;
        }
        else
        {
            if (!current.inSequenceTo(lastcard))
            {
                m_chaos++;
            }
        }
        lastcard = current;
    }
}

void Pile::clear()
{
    memset(cards, 0, MAX_CARDS + 1);
    count = 0;
}

const Pile *Pile::assignLeftCards(QList<Card> &list) const
{
    unsigned char newcards[MAX_CARDS];
    memcpy(newcards, cards, MAX_CARDS);

    for (int index = 0; index < cardCount(); index++)
    {
        if (at(index).is_unknown())
        {
            Card c = list.takeFirst();
            c.set_faceup(at(index).is_faceup());
            newcards[index] = c.raw_value();
        }
    }
    return query_or_insert(newcards, count);
}

const Pile *Pile::replaceAt(int index, const Card &c) const
{
    unsigned char newcards[MAX_CARDS];
    memcpy(newcards, cards, MAX_CARDS);
    newcards[index] = c.raw_value();
    return query_or_insert(newcards, count);
}

int Pile::sequenceOf_(Suit suit) const
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
    return sequenceOf(at(count - 1).suit());
}
