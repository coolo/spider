#include <QCommandLineParser>
#include <QDebug>
#include <QFile>
#include <QMessageLogger>
#include <iostream>
#include <queue>
#include "card.h"
#include "pile.h"
#include "move.h"
#include "deck.h"

class ChaosCompare
{
public:
    bool operator()(Deck *v1, Deck *v2)
    {
        return v1->chaos() > v2->chaos();
    }
};

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
    d.calculateChaos();
    std::priority_queue<Deck *, std::vector<Deck *>, ChaosCompare> list;
    QSet<uint64_t> seen;
    list.push(&d);
    int min_chaos = INT_MAX;
    do
    {
        d = *list.top();
        list.pop();
        //qDebug() << d.chaos();
        QList<Move> moves = d.getMoves();
        if (d.chaos() < min_chaos)
        {
            min_chaos = d.chaos();
            std::cout << std::endl
                      << std::endl
                      << min_chaos << std::endl
                      << d.toString().toStdString();
        }
        for (Move m : moves)
        {
            //std::cout << std::endl << std::endl << d.toString().toStdString();
            //std::cout << d.explainMove(m).toStdString() << std::endl;
            Deck *newdeck = d.applyMove(m);
            uint64_t id = newdeck->id();
            //std::cout << std::endl << std::endl << newdeck->toString().toStdString();
            //std::cout << newdeck->id() << " " << seen.contains(id) << std::endl;
            if (!seen.contains(id))
            {
                seen.insert(id);
                list.push(newdeck);
                //qDebug() << newdeck->chaos() << list.count();
            }
            else
            {
                delete newdeck;
            }
        }
    } while (!list.empty() && min_chaos > 0);
    return 0;
}
