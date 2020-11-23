#include <QCommandLineParser>
#include <QDebug>
#include <QFile>
#include <QMessageLogger>
#include <iostream>
#include "card.h"
#include "pile.h"
#include "move.h"
#include "deck.h"

bool compare_chaos(Deck *v1, Deck *v2)
{
    return v1->chaos() < v2->chaos();
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
    d.calculateChaos();
    QList<Deck *> list;
    QSet<uint64_t> seen;
    list.append(&d);
    int min_chaos = INT_MAX;
    do
    {
        std::sort(list.begin(), list.end(), compare_chaos);
        d = *list.takeFirst();
        QList<Move> moves = d.getMoves();
        if (d.chaos() < min_chaos)
        {
            std::cout << std::endl
                      << std::endl
                      << min_chaos << d.toString().toStdString();
            min_chaos = d.chaos();
        }
        for (Move m : moves)
        {
            //std::cout << std::endl << std::endl << d.toString().toStdString();
            //std::cout << d.explainMove(m).toStdString() << std::endl;
            Deck *newdeck = d.applyMove(m);
            uint64_t id = newdeck->id();
            //std::cout << newdeck->id() << " " << seen.contains(id) << std::endl;
            if (!seen.contains(id))
            {
                seen.insert(id);
                list.append(newdeck);
                //qDebug() << newdeck->chaos() << list.count();
            }
            else
            {
                delete newdeck;
            }
        }
    } while (!list.empty());
    return 0;
}
