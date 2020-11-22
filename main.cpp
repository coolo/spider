#include "SpookyV2.h"
#include <QCommandLineParser>
#include <QDebug>
#include <QFile>
#include <QMessageLogger>
#include <iostream>
#include "card.h"

class Pile
{
public:
    Pile(QString _prefix) { prefix = _prefix; }
    bool addCard(QString token);
    void addCard(const Card &c);
    QString toString();
    QString name() const { return prefix; }
    bool empty() const { return cards.empty(); }
    Card at(int index) const { return cards[index]; }
    size_t cardCount() const { return cards.count(); }
    Pile *remove(int index);
    Pile *copyFrom(Pile *from, int index);
private:
    QString prefix;
    QList<Card> cards;
};

Pile *Pile::copyFrom(Pile *from, int index) {
    Pile *newone = new Pile(prefix);
    newone->cards = cards;
    for (int i = index; i < from->cardCount(); i++ ) {
        qDebug() << "T" << from->at(i).toString();
        newone->addCard(from->at(i));
    }
    return newone;
}

QString Pile::toString()
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
    return newone;
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
    return true;
}

void Pile::addCard(const Card &c)
{
    cards.append(c);
}

struct Move
{
    bool off;
    bool talon;
    int from;
    int to;
    int index;
    Move()
    {
        talon = false;
        off = false;
        from = -1;
        to = -1;
        index = 0;
    }
};

class Deck
{
public:
    Pile *addPile(QString token)
    {
        Pile *p = new Pile(token);
        piles.append(p);
        return p;
    }
    QList<Pile *> piles;
    QList<Move> getMoves();
    QString toString() const
    {
        QString ret;
        for (Pile *p : piles)
        {
            ret += p->toString();
            ret += QStringLiteral("\n");
        }
        return ret;
    }
    QString explainMove(Move m);
    Deck *applyMove(Move m);
};

QList<Move> Deck::getMoves()
{
    QList<Move> ret;
    int from = 0;
    for (; from < 10; from++)
    {
        qDebug() << "Play" << piles[from]->toString();
        if (piles[from]->empty())
            continue;

        int count = piles[from]->cardCount();
        Suit top_suit = piles[from]->at(count - 1).suit;
        int top_rank = int(piles[from]->at(count - 1).rank) + 1;

        while (count >= 0)
        {
            Card current = piles[from]->at(count - 1);
            if (!current.faceup)
                break;
            if (current.suit != top_suit)
                break;
            if (top_rank - 1 != current.rank)
            {
                qDebug() << "stop at" << count << "as ranks not order";
                break;
            }
            top_rank = piles[from]->at(count - 1).rank;

            for (int to = 0; to < 10; to++)
            {
                if (to == from)
                    continue;
                qDebug() << "trying to move " << count << " from " << from << " to " << to;
                size_t to_count = piles[to]->cardCount();
                if (to_count > 0)
                {
                    Card top_card = piles[to]->at(to_count - 1);
                    if (top_card.rank != top_rank + 1)
                    {
                        qDebug() << "no match";
                        continue;
                    }
                }
                ret.append(Move());
                ret.last().from = from;
                ret.last().to = to;
                ret.last().index = count - 1;
            }
        }
    }
    from = 10;
    for (; from < 14; from++)
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
            newone->piles[9 - to]->addCard(c);
        }
        // empty pile
        newone->piles[m.from] = new Pile(newone->piles[m.from]->name());
        return newone;
    }
    newone->piles[m.to] = newone->piles[m.to]->copyFrom(newone->piles[m.from], m.index);
    newone->piles[m.from] = newone->piles[m.from]->remove(m.index);
    return newone;
}

int main(int argc, char **argv)
{
    QCoreApplication app(argc, argv);
    QCoreApplication::setApplicationName("spider");
    QCoreApplication::setApplicationVersion("1.0");

    QCommandLineParser parser;
    parser.setApplicationDescription("Solve Spider games");
    parser.addHelpOption();
    parser.addVersionOption();
    parser.addPositionalArgument("game", "Description of game");

    parser.process(app);

    const QStringList args = parser.positionalArguments();
    if (args.empty())
        return 1;

    QFile file(args[0]);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text))
        return 1;

    QTextStream ts(&file);
    Deck d;
    Pile *current_pile = 0;
    while (!ts.atEnd())
    {
        QString token;
        ts >> token;

        if (token.startsWith("Play") || token.startsWith("Deal") || token.startsWith("Off"))
        {
            current_pile = d.addPile(token);
        }
        else if (!token.isEmpty() && current_pile)
        {
            current_pile->addCard(token);
        }
    }

    QList<Move> moves = d.getMoves();
    std::cout << d.toString().toStdString();
    for (Move m : moves)
    {
        std::cout << d.explainMove(m).toStdString() << std::endl;
        Deck *newdeck = d.applyMove(m);
        std::cout << newdeck->toString().toStdString();
    }
    return 0;
}
