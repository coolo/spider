#include "card.h"
#include "deck.h"
#include "move.h"
#include "pile.h"
#include <QCommandLineParser>
#include <QDebug>
#include <QFile>
#include <QMessageLogger>
#include <iostream>
#include <queue>

class ChaosCompare {
public:
    bool operator()(Deck* v1, Deck* v2)
    {
        return v1->chaos() + v1->moves() > v2->chaos() + v2->moves();
    }
};

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
    Deck* d = new Deck();
    Card cards[104];
    QList<Card> required;
    int game_type = 2;
    for (int suit = 0; suit < 4; suit++) {
        for (int r = Ace; r <= King; r++) {
            Card c;
            c.rank = (Rank)r;
            if (game_type == 2) {
                c.suit = suit % 2 ? Hearts : Spades;
            } else {
                c.suit = Spades;
            }
            required.append(c);
            required.append(c);
        }
    }
    int count = -1;
    while (!ts.atEnd()) {
        QString token;
        ts >> token;

        if (token.startsWith("Play") || token.startsWith("Deal") || token.startsWith("Off")) {
            if (count >= 0)
                d->addPile(cards, count);
            count = 0;
        } else if (!token.isEmpty()) {
            Card c(token);
            required.removeOne(c);
            cards[count++] = c;
        }
    }
    std::random_shuffle(required.begin(), required.end());
    d->addPile(cards, count);
    d->assignLeftCards(required);
    d->calculateChaos();
    Deck orig = *d;
    std::priority_queue<Deck*, std::vector<Deck*>, ChaosCompare> lists[6];
    QMap<uint64_t, int> seen;
    lists[d->leftTalons()].push(d);
    int min_chaos = INT_MAX;
    int roundrobin = 0;
    do {
        if (lists[roundrobin].empty()) 
            goto nextlist;

        d = lists[roundrobin].top();
        lists[roundrobin].pop();
        //qDebug() << d.chaos();
        QList<Move> moves = d->getMoves();
        if (d->chaos() < min_chaos) {
            min_chaos = d->chaos();
            std::cout << std::endl
                      << std::endl
                      << min_chaos << " " << d->moves() << std::endl
                      << d->toString().toStdString();
            if (!min_chaos) {
                int counter = 1;
                for (Move m : d->order) {
                    //std::cout << orig.toString().toStdString() << std::endl;
                    if (!m.off)
                        std::cout << QString("%1").arg(counter++).toStdString() << " " << orig.explainMove(m).toStdString() << std::endl;
                    orig = *orig.applyMove(m, true);
                }
            }
        }
        for (Move m : moves) {
            //std::cout << std::endl << std::endl << d->chaos() << std::endl << d->toString().toStdString();
            //std::cout << d->explainMove(m).toStdString() << std::endl;
            Deck* newdeck = d->applyMove(m);
            //std::cout << "new chaos " << newdeck->chaos() << std::endl << newdeck->toString().toStdString() << std::endl;
            uint64_t id = newdeck->id();
            //std::cout << std::endl << std::endl << newdeck->toString().toStdString();
            //std::cout << newdeck->id() << " " << seen.contains(id) << std::endl;
            if (!seen.contains(id)) {
                seen.insert(id, newdeck->moves());

                lists[newdeck->leftTalons()].push(newdeck);
                //qDebug() << newdeck->chaos() << list.size();
                const int max_elements = 300000;
                if (lists[roundrobin].size() > max_elements) {
                    qDebug() << "reduce" << seen.size();
                    std::cout << std::endl
                              << std::endl
                              << newdeck->toString().toStdString();
                    std::vector<Deck*> tmp;
                    for (int i = 0; i < max_elements / 2; i++) {
                        tmp.push_back(lists[roundrobin].top());
                        lists[roundrobin].pop();
                    }
                    while (!lists[roundrobin].empty()) {
                        delete lists[roundrobin].top();
                        lists[roundrobin].pop();
                    }
                    lists[roundrobin] = std::priority_queue<Deck*, std::vector<Deck*>, ChaosCompare>();
                    std::vector<Deck*>::iterator it = tmp.begin();
                    for (; it != tmp.end(); it++)
                        lists[roundrobin].push(*it);
                }
            } else {
                if (newdeck->moves() < seen[id])
                    seen[id] = newdeck->moves();
                delete newdeck;
            }
        }
        delete d;

    nextlist: 
        roundrobin = rand() % 6;
    } while (min_chaos > 0);
    for (int i = 0; i < 6; i++) {
        while (!lists[i].empty()) {
            delete lists[i].top();
            lists[i].pop();
        }
    }
    return 0;
}
