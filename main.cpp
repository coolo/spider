#include "card.h"
#include "deck.h"
#include "move.h"
#include "pile.h"
#include <time.h>
#include <QCommandLineParser>
#include <QDebug>
#include <QFile>
#include <QMessageLogger>
#include <iostream>
#include <queue>

class ChaosCompare
{
public:
    bool operator()(Deck *v1, Deck *v2)
    {
        return v1->chaos() + v1->moves() > v2->chaos() + v2->moves();
    }
};

typedef std::priority_queue<Deck *, std::vector<Deck *>, ChaosCompare> DeckList;

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
    Deck *d = new Deck();
    Card cards[104];
    QList<Card> required;
    int game_type = 2;
    for (int suit = 0; suit < 4; suit++)
    {
        for (int r = Ace; r <= King; r++)
        {
            Card c;
            c.set_rank(Rank(r));
            if (game_type == 2)
            {
                c.set_suit(suit % 2 ? Hearts : Spades);
            }
            else
            {
                c.set_suit(Spades);
            }
            required.append(c);
            required.append(c);
        }
    }
    int count = -1;
    while (!ts.atEnd())
    {
        QString token;
        ts >> token;
        if (token.startsWith("#"))
        {
            ts.readLine();
            continue;
        }

        if (token.startsWith("Play") || token.startsWith("Deal") || token.startsWith("Off"))
        {
            if (count >= 0)
                d->addPile(cards, count);
            count = 0;
        }
        else if (!token.isEmpty())
        {
            if (token.length() == 6)
            {
                Card first(token.mid(0, 2));
                Q_ASSERT(token.mid(2, 2) == "..");
                //if (token.mid(2,4))
                Card last(token.mid(4, 2));
                while (first.rank() >= last.rank())
                {
                    assert(required.contains(first));
                    required.removeOne(first);
                    cards[count++] = first;
                    first.set_rank(Rank(first.rank() - 1));
                }
            }
            else
            {
                Card c(token);
                if (d->pilesAdded() == 15)
                {
                    for (int rank = Ace; rank <= King; rank++)
                    {
                        c.set_rank(Rank(rank));
                        assert(required.contains(c));
                        required.removeOne(c);
                    }
                }
                else
                {
                    if (!c.is_unknown())
                    {
                        if (!required.contains(c))
                        {
                            qDebug() << "too many" << c;
                            assert(required.contains(c));
                        }
                    }
                    required.removeOne(c);
                }
                cards[count++] = c;
            }
        }
    }
    if (!required.empty())
    {
        for (int i = 0; i < required.size(); i++)
            required[i].set_unknown(false);
        qDebug() << "Required left:" << required;
    }
    // take this with standard seed
    std::random_shuffle(required.begin(), required.end());
    srand(time(0));
    d->addPile(cards, count);
    d->assignLeftCards(required);
    if (!required.empty())
    {
        for (int i = 0; i < required.size(); i++)
            required[i].set_unknown(false);
        qDebug() << "Required left:" << required;
    }
    Q_ASSERT(required.empty());
    d->calculateChaos();
    std::cout << d->toString().toStdString() << std::endl;
    Deck orig = *d;
    DeckList lists[6];
    QMap<uint64_t, int> seen;

    QList<Move> moves = d->getMoves();
    for (Move m : moves)
    {
        std::cout << d->explainMove(m).toStdString() << std::endl;
        Deck *newdeck = d->applyMove(m);
        std::cout << newdeck->toString().toStdString() << std::endl;
    }

    return 0;
}
