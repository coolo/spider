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
    d.makeEmpty();
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
    int piles = -1;
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
            piles++;
        }
        else if (!token.isEmpty())
        {
            if (token.length() == 6)
            {
                Card first(token.mid(0, 2).toStdString());
                Q_ASSERT(token.mid(2, 2) == "..");
                //if (token.mid(2,4))
                Card last(token.mid(4, 2).toStdString());
                while (first.rank() >= last.rank())
                {
                    assert(required.contains(first));
                    required.removeOne(first);
                    d.addCard(piles, first);
                    first.set_rank(Rank(first.rank() - 1));
                }
            }
            else
            {
                Card c(token.toStdString());
                if (piles == 15)
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
                            std::cerr << "too many " << c.toString() << std::endl;
                            exit(1);
                        }
                    }
                    required.removeOne(c);
                }
                d.addCard(piles, c);
            }
        }
    }
    // take this with standard seed
    std::random_shuffle(required.begin(), required.end());
    srand(time(0));
    d.assignLeftCards(required);

    if (!required.empty())
    {
        for (int i = 0; i < required.size(); i++)
            required[i].set_unknown(false);
        std::cerr << "Required left: [ ";
        for (Card c : required)
        {
            std::cerr << c.toString() << " ";
        }
        std::cerr << " ]" << std::endl;
        exit(1);
    }
    if (d.shortestPath(500, false) > 0)
    {
        qDebug() << "WON";
        int counter = 1;

        Deck orig = d;
        for (Move m : d.getWinMoves())
        {
            //std::cout << orig.toString().toStdString() << std::endl;
            if (!m.off)
                std::cout << QString("%1").arg(counter++).toStdString() << " " << orig.explainMove(m).toStdString() << std::endl;
            orig.applyMove(m, orig, true);
        }
    }

    return 0;
}
