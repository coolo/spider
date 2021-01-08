#include "deck.h"
#include "move.h"
#include "pile.h"
#include "card.h"
#include "SpookyV2.h"
#include <QList>
#include <QFile>
#include <QDebug>
#include <iostream>

struct WeightedDeck
{
    uchar left_talons;
    int chaos;
    int64_t id;
    uchar in_off;
    uchar free_plays;
    uchar playable;
    Deck deck;

    bool operator<(const WeightedDeck &rhs) const;
    void update(uint64_t hash);
};

void WeightedDeck::update(uint64_t hash)
{
    left_talons = deck.leftTalons();
    id = hash;
    in_off = deck.inOff();
    free_plays = deck.freePlays();
    playable = deck.playableCards();
    chaos = deck.chaos();
}

// smaller is better!
bool WeightedDeck::operator<(const WeightedDeck &rhs) const
{
    if (chaos != rhs.chaos)
    {
        // smaller chaos is better
        return chaos < rhs.chaos;
    }
    int ready1 = playable + in_off + free_plays;
    int ready2 = rhs.playable + rhs.in_off + rhs.free_plays;
    if (ready1 != ready2)
    {
        // larger values are better
        return ready1 > ready2;
    }

    // once we are in straight win mode, we go differently
    if (chaos == 0)
    {
        int free1 = free_plays;
        int free2 = rhs.free_plays;

        if (free1 != free2)
        {
            // more free is better
            return free1 > free2;
        }
        // if the number of empty plays is equal, less in the off
        // is actually a benefit (more strongly ordered)
        int off1 = in_off;
        int off2 = rhs.in_off;
        if (off1 != off2)
        {
            return off1 < off2;
        }
    }
    // give a reproducible sort order, but std::sort doesn't give
    // guarantess for equal items, so prefer them being different
    return id < rhs.id;
}

void Deck::getMoves(QVector<Move> &moves) const
{
    moves.clear();
    if (moves_index >= MAX_MOVES - 1)
    {
        return;
    }
    int next_talon = -1;
    for (int i = 0; i < 5; i++)
    {
        if (!talon[i].empty())
        {
            next_talon = i;
            break;
        }
    }

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
                moves.clear();
                moves.append(Move::toOff(from, index));
                return;
            }
            int broken_sequence = 0;
            if (index > 0)
            {
                Card next_card = play[from].at(index - 1);
                if (current.inSequenceTo(next_card))
                {
                    broken_sequence = play[from].cardCount() - index;
                }
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
                    // make sure we only enlarge sequences
                    if (broken_sequence > 0)
                    {
                        if (play[to].sequenceOf(top_suit) + broken_sequence <= play[from].sequenceOf(top_suit))
                        {
                            continue;
                        }
                    }
                }
                else if (moved_to_empty)
                {
                    if (next_talon < 0)
                        continue;
                }
                else
                {
                    // while talons are there, optimisations are evil
                    // but in end game we have more options
                    if (next_talon < 0)
                    {
                        if (index == 0)
                        {
                            // forbid moves between empty cells once the talons are gone
                            continue;
                        }
                        // there is no plausible reason to split up sequences in end game
                        if (broken_sequence > 0)
                        {
                            continue;
                        }
                    }
                    moved_to_empty = true;
                }

                moves.append(Move::regular(from, to, index));
            }
            index--;
        }
    }

    if (!one_is_empty && next_talon >= 0)
    {
        moves.append(Move::fromTalon(next_talon));
    }
}

void Deck::update(const Deck &other)
{
    memcpy(moves, other.moves, sizeof(Move) * MAX_MOVES);
    moves_index = other.moves_index;
    for (int i = 0; i < 10; i++)
        play[i].clone(other.play[i]);
    for (int i = 0; i < 5; i++)
        talon[i].clone(other.talon[i]);
    off.clone(other.off);
}

QVector<Move> Deck::getWinMoves() const
{
    QVector<Move> res;
    for (int i = 0; i < moves_index; i++)
    {
        res.append(moves[i]);
    }
    return res;
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
    std::string fromCard = play[m.from].at(m.index).toString();
    std::string toCard = "Empty";
    if (play[m.to].cardCount() > 0)
        toCard = play[m.to].at(play[m.to].cardCount() - 1).toString();
    return QString("Move %1 cards from %2 to %3 - %4->%5").arg(play[m.from].cardCount() - m.index).arg(m.from + 1).arg(m.to + 1).arg(QString::fromStdString(fromCard)).arg(QString::fromStdString(toCard));
}

void Deck::applyMove(const Move &m, Deck &newdeck, bool stop)
{
    // newdeck could be this - but no worries
    newdeck = *this;
    newdeck.moves[moves_index] = m;
    newdeck.moves_index = moves_index + 1;

    if (m.talon)
    {

        for (int to = 0; to < 10; to++)
        {
            Card c = newdeck.talon[m.from].at(to);
            c.set_faceup(true);
            newdeck.play[to].addCard(c);
        }
        // empty pile
        newdeck.talon[m.from].clear();
    }
    else if (m.off)
    {
        Card c = newdeck.play[m.from].at(newdeck.play[m.from].cardCount() - 13);
        newdeck.off.addCard(c);
        newdeck.play[m.from].remove(m.index);
    }
    else
    {
        newdeck.play[m.to].copyFrom(newdeck.play[m.from], m.index);
        newdeck.play[m.from].remove(m.index);
        if (stop && m.index > 0 && newdeck.play[m.from].at(m.index - 1).is_unknown())
        {
            std::cout << "What's up?" << std::endl;
            std::string line;
            std::getline(std::cin, line);
            Card c(line);
            newdeck.play[m.from].replaceAt(m.index - 1, c);
            QFile file("tmp");
            file.open(QIODevice::WriteOnly);
            file.write(newdeck.toString().toUtf8());
            file.close();
            exit(1);
        }
    }
}

QString Deck::toString() const
{
    QString ret;
    int counter = 0;
    for (int i = 0; i < 10; i++)
    {
        ret += QString("Play%1:").arg(i);
        ret += QString::fromStdString(play[i].toString());
        ret += QStringLiteral("\n");
    }

    for (int i = 0; i < 5; i++)
    {
        ret += QString("Deal%1:").arg(i);
        ret += QString::fromStdString(talon[i].toString());
        ret += QStringLiteral("\n");
        counter++;
    }

    ret += "Off:";
    ret += QString::fromStdString(off.toString());
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

int Deck::chaos() const
{
    int chaos = 0;
    for (int i = 0; i < 10; i++)
    {
        chaos += play[i].chaos();
    }
    for (int i = 0; i < 5; i++)
    {
        if (!talon[i].empty())
        {
            chaos += 11;
        }
    }
    return chaos;
}

int Deck::shortestPath(int cap, bool debug)
{
    int depth = 1;
    moves_index = 0;

    Deck *unvisited = new Deck[6 * cap];
    int unvisited_count[6] = {0, 0, 0, 0, 0, 0};
    int unvisited_count_total = 0;
    unvisited[unvisited_count_total++].update(*this);

    QSet<uint64_t> seen;
    const int max_new_unvisited = cap * 6 * 30;
    WeightedDeck *new_unvisited = new WeightedDeck[max_new_unvisited];
    int new_unvisited_counter = 0;
    QVector<Move> current_moves;
    while (true)
    {
        for (int i = 0; i < unvisited_count_total; i++)
        {
            // std::cout << deck->toString().toStdString() << std::endl;
            unvisited[i].getMoves(current_moves);
            for (Move m : current_moves)
            {
                //std::cout << deck->explainMove(m).toStdString() << std::endl;
                unvisited[i].applyMove(m, new_unvisited[new_unvisited_counter].deck);
                //std::cout << newdeck.toString().toStdString() << std::endl;
                uint64_t hash = new_unvisited[new_unvisited_counter].deck.id();
                if (!seen.contains(hash))
                {
                    new_unvisited[new_unvisited_counter++].update(hash);
                    seen.insert(hash);
                    if (max_new_unvisited == new_unvisited_counter)
                    {
                        std::cerr << "Too many unvisted " << new_unvisited_counter << std::endl;
                        exit(1);
                    }
                }
            }
        }
        for (int lt = 0; lt <= 5; lt++)
            unvisited_count[lt] = 0;

        unvisited_count_total = 0;

        if (new_unvisited_counter == 0)
            break;

        bool printed = false;
        std::sort(new_unvisited, new_unvisited + new_unvisited_counter);
        for (int i = 0; i < new_unvisited_counter; i++)
        {
            if (!printed)
            {
                std::cout << "DEPTH " << depth << " " << new_unvisited_counter << " chaos: " << new_unvisited[i].chaos << " " << int(new_unvisited[i].playable) << std::endl;
                printed = true;
            }
            if (new_unvisited[i].in_off == 104)
            {
                memcpy(moves, new_unvisited[i].deck.moves, sizeof(Move) * MAX_MOVES);
                moves_index = new_unvisited[i].deck.moves_index;
                delete[] unvisited;
                delete[] new_unvisited;
                return depth;
            }
            int lt = new_unvisited[i].left_talons;
            if (unvisited_count[lt] < cap)
            {
                unvisited[unvisited_count_total++].update(new_unvisited[i].deck);
                unvisited_count[lt]++;
            }
        }

        new_unvisited_counter = 0;
        depth += 1;
    }
    delete[] unvisited;
    delete[] new_unvisited;
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

int Deck::playableCards() const
{
    int result = 0;
    for (int i = 0; i < 10; i++)
        result += play[i].playableCards();
    return result;
}

int Deck::inOff() const
{
    return off.cardCount() * 13;
}

int Deck::freePlays() const
{
    int result = 0;
    for (int i = 0; i < 10; i++)
    {
        if (play[i].empty())
        {
            result++;
        }
    }
    return result;
}

bool Deck::isWon() const
{
    return off.cardCount() == 8;
}