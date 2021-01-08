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
    Move(bool _off, bool _talon, int _from, int _to, int _index)
    {
        off = _off;
        talon = _talon;
        from = _from;
        to = _to;
        index = _index;
    }
    static Move fromTalon(int talon)
    {
        return Move(false, true, talon, 0, 0);
    }
};

#endif