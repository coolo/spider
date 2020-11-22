#include "SpookyV2.h"
#include <QCommandLineParser>
#include <QDebug>
#include <QFile>

#include "card.h"

class Pile
{
public:
    Pile(QString _prefix) { prefix = _prefix; }
    bool addCard(QString token);
    QString toString();

private:
    QString prefix;
    QList<Card> cards;
};

QString Pile::toString()
{
    QString ret = prefix;
    for (Card c : cards)
    {
        ret += " " + c.toString();
    }
    return ret;
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
    for (Pile *p : d.piles)
    {
        qDebug() << p->toString();
    }

    return 0;
}
