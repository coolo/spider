#include "deck.h"
#include "move.h"
#include "pile.h"
#include "card.h"
#include "SpookyV2.h"
#include <QList>
#include <QFile>
#include <QDebug>
#include <iostream>

QList<Move> Deck::getMoves()
{
    bool talons_done = true;
    for (int i = 0; i < 5; i++)
    {
        if (!talon[i].empty())
        {
            talons_done = false;
            break;
        }
    }

    QList<Move> ret;
    int from = 0;
    bool one_is_empty = false;
    for (; from < 10; from++)
    {
        //qDebug() << "Play" << piles[from]->toString();
        if (play[from].empty())
        {
            one_is_empty = true;
            continue;
        }

        int index = play[from].cardCount() - 1;
        Suit top_suit = play[from].at(index).suit();
        int top_rank = int(play[from].at(index).rank()) - 1;

        while (index >= 0)
        {
            Card current = play[from].at(index);
            if (!current.is_faceup())
                break;
            if (current.suit() != top_suit)
                break;
            if (top_rank + 1 != current.rank())
                break;
            top_rank = play[from].at(index).rank();

            if (play[from].cardCount() - index == 13)
            {
                ret.clear();
                ret.append(Move());
                ret.last().from = from;
                ret.last().to = 0;
                ret.last().off = true;
                ret.last().index = index;
                return ret;
            }
            bool moved_to_empty = false;
            for (int to = 0; to < 10; to++)
            {
                if (to == from)
                    continue;
                //qDebug() << "trying to move " << (piles[from]->cardCount() - index) << " from " << from << " to " << to;
                int to_count = play[to].cardCount();
                if (to_count > 0)
                {
                    Card top_card = play[to].at(to_count - 1);
                    if (top_card.rank() != top_rank + 1)
                        continue;
                }
                else if (moved_to_empty)
                {
                    if (talons_done)
                        continue;
                }
                else
                {
                    moved_to_empty = true;
                }

                ret.append(Move());
                ret.last().from = from;
                ret.last().to = to;
                ret.last().index = index;
            }
            index--;
        }
    }

    if (!one_is_empty)
    {
        from = 0;
        for (; from < 5; from++)
        {
            if (!talon[from].empty())
            {
                ret.append(Move());
                ret.last().from = from;
                ret.last().talon = true;
                break;
            }
        }
    }
    return ret;
}

Deck::Deck(const Deck &other)
{
    m_moves = other.m_moves;
    order = other.order;
    for (int i = 0; i < 10; i++)
        play[i].clone(other.play[i]);
    for (int i = 0; i < 5; i++)
        talon[i].clone(other.talon[i]);
    off.clone(other.off);
    m_chaos = other.m_chaos;
}

QString Deck::explainMove(Move m)
{
    if (m.talon)
    {
        return "Draw another talon";
    }
    if (m.off)
    {
        return QString("Move a sequence from %1 to the off").arg(m.from + 1);
    }
    QString fromCard = play[m.from].at(m.index).toString();
    QString toCard = "Empty";
    if (play[m.to].cardCount() > 0)
        toCard = play[m.to].at(play[m.to].cardCount() - 1).toString();
    return QString("Move %1 cards from %2 to %3 - %4->%5").arg(play[m.from].cardCount() - m.index).arg(m.from + 1).arg(m.to + 1).arg(fromCard).arg(toCard);
}

Deck *Deck::applyMove(Move m, bool stop)
{
    Deck *newone = new Deck(*this);

    if (!m.off)
    {
        newone->m_moves += 1;
        //qDebug() << newone->m_moves;
    }
    newone->order.append(m);
    if (m.talon)
    {

        for (int to = 0; to < 10; to++)
        {
            Card c = newone->talon[m.from].at(to);
            c.set_faceup(true);
            newone->play[to].addCard(c);
        }
        // empty pile
        newone->talon[m.from].clear();
    }
    else if (m.off)
    {
        Card c = newone->play[m.from].at(newone->play[m.from].cardCount() - 13);
        newone->off.addCard(c);
        newone->play[m.from].remove(m.index);
    }
    else
    {
        newone->play[m.to].copyFrom(newone->play[m.from], m.index);
        newone->play[m.from].remove(m.index);
        if (stop && m.index > 0 && newone->play[m.from].at(m.index - 1).is_unknown())
        {
            std::cout << "What's up?" << std::endl;
            std::string line;
            std::getline(std::cin, line);
            Card c(QString::fromStdString(line));
            newone->play[m.from].replaceAt(m.index - 1, c);
            QFile file("tmp");
            file.open(QIODevice::WriteOnly);
            file.write(newone->toString().toUtf8());
            file.close();
            exit(1);
        }
    }
    newone->calculateChaos();
    return newone;
}

QString Deck::toString() const
{
    QString ret;
    int counter = 0;
    for (int i = 0; i < 10; i++)
    {
        ret += QString("Play%1:").arg(i);
        ret += play[i].toString();
        ret += QStringLiteral("\n");
    }

    for (int i = 0; i < 5; i++)
    {
        ret += QString("Deal%1:").arg(i);
        ret += talon[i].toString();
        ret += QStringLiteral("\n");
        counter++;
    }

    ret += "Off:";
    ret += off.toString();
    ret += QStringLiteral("\n");

    return ret;
}

uint64_t Deck::id() const
{
    // TODO: ignore off
    uchar buffer[15 * MAX_CARDS];
    for (int i = 0; i < 10; i++)
        memcpy(buffer + i * MAX_CARDS, play[i].cardsPtr(), MAX_CARDS);
    for (int i = 0; i < 5; i++)
        memcpy(buffer + (i + 10) * MAX_CARDS, talon[i].cardsPtr(), MAX_CARDS);

    return SpookyHash::Hash64(&buffer, 15 * MAX_CARDS, 1);
}

void Deck::assignLeftCards(QList<Card> &list)
{
    for (int i = 0; i < 10; i++)
        play[i].assignLeftCards(list);
    for (int i = 0; i < 5; i++)
        talon[i].assignLeftCards(list);
}

int Deck::free_talons() const
{
    int talons = 0;
    for (int i = 0; i < 5; i++)
    {
        if (!talon[i].empty())
        {
            talons++;
        }
    }
    return talons;
}

void Deck::calculateChaos()
{
    m_chaos = 0;
    for (int i = 0; i < 10; i++)
    {
        m_chaos += play[i].chaos();
    }
    for (int i = 0; i < 5; i++)
    {
        if (!talon[i].empty())
        {
            m_chaos += 11;
        }
    }
}

int Deck::leftTalons() const
{
    int talons = 0;

    for (int i = 0; i < 5; i++)
    {
        if (!talon[i].empty())
        {
            talons++;
        }
    }
    return talons;
}

int Deck::shortestPath(int cap, bool debug)
{
    int depth = 0;

    QVector<Deck> unvisited[6];
    unvisited[free_talons()].append(Deck(*this));
    QSet<uint64_t> seen;
    QVector<Deck> new_unvisited;

    while (true)
    {
        for (int i = 0; i <= 5; i++)
        {
            for (auto deck = unvisited[i].begin(); deck != unvisited[i].end(); deck++)
            {
                //std::cout << deck->toString().toStdString() << std::endl;
                QList<Move> moves = deck->getMoves();
                for (Move m : moves)
                {
                    //std::cout << deck->explainMove(m).toStdString() << std::endl;
                    Deck *newdeck = deck->applyMove(m);
                    uint64_t hash = newdeck->id();
                    if (!seen.contains(hash))
                    {
                        new_unvisited.append(*newdeck);
                        seen.insert(hash);
                    }
                    delete newdeck;
                }
            }
            unvisited[i].clear();
        }
        qDebug() << "DEPTH" << depth << new_unvisited.length();
        if (new_unvisited.empty())
            break;

        std::sort(new_unvisited.begin(), new_unvisited.end());
        QVector<Deck>::const_reverse_iterator it = new_unvisited.crbegin();
        for (; it != new_unvisited.crend(); ++it)
        {

            if (it->is_won())
            {
                qDebug() << "WIN";
                return depth;
            }
            int lt = it->leftTalons();
            if (unvisited[lt].length() < cap)
            {
                unvisited[lt].append(*it);
            }
        }

        new_unvisited.clear();
        depth += 1;
    }
    return -1 * depth;
}

void Deck::addCard(int index, const Card &c)
{
    if (index < 10)
    {
        play[index].addCard(c);
    }
    else
    {
        talon[index - 10].addCard(c);
    }
}

bool Deck::operator<(const Deck &rhs) const
{
    return false;
}

bool Deck::is_won() const
{
    return off.cardCount() == 8;
}