#include <QString>

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
    Suit char2suit(char c);
    Rank char2rank(char c);
};
