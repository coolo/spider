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

struct WeightedMove
{
    int moves;
    Move move;

    bool operator<(const WeightedMove &rhs) const { return moves > rhs.moves; }
};

int recursion(const Deck &orig, int cap, int best_sofar)
{
    std::vector<Move> current_moves;
    orig.getMoves(current_moves);
    //std::cout << orig.toString() << std::endl;
    Deck newdeck;
    std::priority_queue<WeightedMove> queue;
    for (const Move &m : current_moves)
    {
        orig.applyMove(m, newdeck);
        WeightedMove w;
        w.move = m;
        w.moves = newdeck.shortestPath(cap, false);
        if (w.moves < 0) // no win found - end of story
            continue;
        queue.push(w);
    }

    while (!queue.empty())
    {
        // we allow a certain slip away from the perfect path - as shortestPath isn't perfect
        if (queue.top().moves + orig.getWinMovesCount() > best_sofar + 5)
        {
            queue.pop();
            continue;
        }
        if (queue.top().moves + orig.getWinMovesCount() < best_sofar)
        {
            std::cout << queue.top().moves << " + " << orig.getWinMovesCount() << " = "
                      << queue.top().moves + orig.getWinMovesCount() << " " << best_sofar << std::endl;
            best_sofar = queue.top().moves + orig.getWinMovesCount();
        }
        if (queue.top().moves < 20)
        {
            // let's trust our shortestPath on this one - and stop recursion
            return best_sofar;
        }

        orig.applyMove(queue.top().move, newdeck);
        best_sofar = recursion(newdeck, cap, best_sofar);
        queue.pop();
    }
    return best_sofar;
}

int main(int argc, char **argv)
{
    int c;
    int digit_optind = 0;
    const int default_cap = 500;
    int cap = default_cap;
    int game_type = 2;
    bool test_recursion = false;

    while (1)
    {
        int this_option_optind = optind ? optind : 1;
        int option_index = 0;
        static struct option long_options[] = {
            {"debug", no_argument, 0, 'd'},
            {"cap", required_argument, 0, 'c'},
            {0, 0, 0, 0}};

        c = getopt_long(argc, argv, "dc:r",
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

        case 'r':
            test_recursion = true;
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
    std::vector<Card> required = d.parse(game_type, filename);
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

    if (test_recursion)
    {
        recursion(d, cap, MAX_MOVES);
        return 0;
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
