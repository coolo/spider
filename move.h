#ifndef _MOVE_H_
#define _MOVE_H_ 1

struct Move
{
    bool off;
    bool talon;
    int from;
    int to;
    int index;
    Move()
    {
        talon = false;
        off = false;
        from = -1;
        to = -1;
        index = 0;
    }
};

#endif