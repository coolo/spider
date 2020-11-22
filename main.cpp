#include "SpookyV2.h"
#include <QCommandLineParser>
#include <QDebug>
#include <QFile>

void process_line(const QByteArray& line)
{
    qDebug() << "Date:" << line;
}

enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds
};
enum Rank {
    Ace,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King
};

struct Card {
    bool faceup;
    Suit suit;
    Rank rank;

    QString toString() const;
};

QString Card::toString() const
{
    QString ret;
    switch (rank) {
    case Ace:
        ret += 'A';
        break;
    case Jack:
        ret += 'J';
        break;
    case Queen:
        ret += 'Q';
        break;
    case King:
        ret += 'K';
        break;
    case Ten:
        ret += 'T';
        break;
    default:
        if (rank < 2 || rank > 9)
            exit(1);
        ret += ('0' + rank);
        break;
    }
    switch (suit) {
    case Spades:
        ret += "S";
        break;
    case Hearts:
        ret += "H";
        break;
    case Diamonds:
        ret += "D";
        break;
    case Clubs:
        ret += "C";
        break;
    default:
        qDebug() << "Invalid suit " << suit;
        exit(1);
    }
    return ret;
}

class Pile {
public:
    Pile(QString _prefix) { prefix = _prefix; }
    bool addCard(QString token);

private:
    QString prefix;
    Suit char2suit(char c);
    Rank char2rank(char c);
};

Suit Pile::char2suit(char c)
{
    switch (c) {
    case 'S':
        return Spades;
    case 'H':
        return Hearts;
    case 'D':
        return Diamonds;
    case 'C':
        return Clubs;
    }
    qDebug() << "No map for " << c;
    exit(1);
    return Spades;
}

Rank Pile::char2rank(char c)
{
    switch (c) {
    case 'K':
        return King;
    case 'Q':
        return Queen;
    case 'A':
        return Ace;
    case 'T':
        return Ten;
    case 'J':
        return Jack;
    case '2':
        return Two;
    case '3':
        return Three;
    case '4':
        return Four;
    case '5':
        return Five;
    case '6':
        return Six;
    case '7':
        return Seven;
    case '8':
        return Eight;
    case '9':
        return Nine;
    }
    qDebug() << "No map for " << c;
    exit(1);
    return Ace;
}

bool Pile::addCard(QString token)
{
    Card newone;
    newone.faceup = token.startsWith('|');
    if (newone.faceup) {
        token.remove(0, 1);
    }
    newone.rank = char2rank(token[0].toLatin1());
    newone.suit = char2suit(token[1].toLatin1());
    return true;
}

int main(int argc, char** argv)
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
    Pile* current_pile = 0;
    while (!ts.atEnd()) {
        QString token;
        ts >> token;

        if (token.startsWith("Play") || token.startsWith("Deal") || token.startsWith("Off")) {
            current_pile = new Pile(token);
        } else if (!token.isEmpty() && current_pile) {
            current_pile->addCard(token);
        }
    }

    return 0;
}
