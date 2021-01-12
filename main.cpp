#include "card.h"
#include "deck.h"
#include "move.h"
#include "pile.h"
#include <time.h>
#include <iostream>
#include <fstream>
#include <queue>
#include <algorithm>
#include <getopt.h>
#include <cassert>

bool removeOne(std::vector<Card> &cards, const Card &c)
{
    std::vector<Card>::iterator position = std::find(cards.begin(), cards.end(), c);
    if (position != cards.end())
    {
        cards.erase(position);
        return true;
    }
    else
    {
        if (!c.is_unknown())
        {
            std::cerr << "too many " << c.toString() << std::endl;
            exit(1);
        }
        return false;
    }
}

int main(int argc, char **argv)
{
    int c;
    int digit_optind = 0;
    const int default_cap = 500;
    int cap = default_cap;

    while (1)
    {
        int this_option_optind = optind ? optind : 1;
        int option_index = 0;
        static struct option long_options[] = {
            {"debug", no_argument, 0, 'd'},
            {"cap", required_argument, 0, 'c'},
            {0, 0, 0, 0}};

        c = getopt_long(argc, argv, "dc:",
                        long_options, &option_index);
        if (c == -1)
            break;

        switch (c)
        {
        case 0:
            printf("option %s", long_options[option_index].name);
            if (optarg)
                printf(" with arg %s", optarg);
            printf("\n");
            break;

        case 'd':
            printf("option d\n");
            break;

        case 'c':
            cap = atoi(optarg);
            if (cap == 0)
                cap = default_cap;
            break;

        case '?':
            break;

        default:
            printf("?? getopt returned character code 0%o ??\n", c);
        }
    }

    if (optind + 1 != argc)
    {
        printf("Require exactly one filename\n");
        return 1;
    }

    std::string filename = argv[optind++];

    Deck d;
    d.makeEmpty();
    std::vector<Card> required;
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
            required.push_back(c);
            required.push_back(c);
        }
    }

    std::ifstream file(filename);
    int piles = -1;
    while (file)
    {
        std::string token;
        file >> token;
        if (token.find('#') == 0)
        {
            std::string line;
            std::getline(file, line);
            continue;
        }

        if (token.find("Play") == 0 || token.find("Deal") == 0 || token.find("Off") == 0)
        {
            piles++;
        }
        else if (!token.empty())
        {
            if (token.length() == 6)
            {
                Card first(token.substr(0, 2));
                assert(token.mid(2, 2) == "..");
                //if (token.mid(2,4))
                Card last(token.substr(4, 2));
                while (first.rank() >= last.rank())
                {
                    removeOne(required, first);
                    d.addCard(piles, first);
                    first.set_rank(Rank(first.rank() - 1));
                }
            }
            else
            {
                Card c(token);
                if (piles == 15)
                {
                    for (int rank = Ace; rank <= King; rank++)
                    {
                        c.set_rank(Rank(rank));
                        removeOne(required, c);
                    }
                }
                else
                {
                    removeOne(required, c);
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
    if (d.shortestPath(cap, false) > 0)
    {
        int counter = 1;
        Deck orig = d;
        for (const Move &m : d.getWinMoves())
        {
            //std::cout << orig.toString() << std::endl;
            if (!m.off)
                std::cout << counter++ << " " << orig.explainMove(m) << std::endl;
            orig.applyMove(m, orig, true);
        }
    }

    return 0;
}
