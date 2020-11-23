#include "SpookyV2.h"
#include <QCommandLineParser>
#include <QDebug>
#include <QFile>
#include <QMessageLogger>
#include <iostream>
#include "card.h"
#include "pile.h"
#include "move.h"
#include "deck.h"

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
    QList<Move> moves;
    do
    {
        moves = d.getMoves();
        std::random_shuffle(moves.begin(), moves.end());
        for (Move m : moves)
        {
            
            //std::cout << std::endl << std::endl << d.toString().toStdString();
            //std::cout << d.explainMove(m).toStdString() << std::endl;
            Deck *newdeck = d.applyMove(m);
            //std::cout << newdeck->toString().toStdString();
            d = *newdeck;
            break;
        }
    } while (!moves.isEmpty());
    return 0;
}
