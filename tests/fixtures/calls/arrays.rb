[]
[1, 2, 3]
[1, [2, 3]]
[1, 2, *a]
[*a, 1]
[*a]
[[1, 2], [3, 4]]
[{ a: 1 }, { b: 2 }]
[1, 2]
[
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    eeeeee,
]
[
    [aaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbb],
    [cccccccccccccccccccccc, dddddddddddddddd, eeeeee],
]
[
    aaaaaaaaaaaaaaaaaaaa
        .bbbbbbbbbbbbbbbbbbbbbbbb
        .cccccccccccccccccccccc
        .dddddddddddddddd
        .eeeeeeeeeeee
        .fff,
    2,
]
[
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    { f: 1 },
]
%w[a b c]
%i[a b c]
%w[]
%i[]
%w[a b]
%i[a b]
%w[a b]
%W[a b]
%I[a b]
%w[]
%i[]
%w[
    aaaaaaaaaaaaaaaaaaaa
    bbbbbbbbbbbbbbbbbbbbbbbb
    cccccccccccccccccccccc
    dddddddddddddddd
    eeeeeee
]
%i[
    aaaaaaaaaaaaaaaaaaaa
    bbbbbbbbbbbbbbbbbbbbbbbb
    cccccccccccccccccccccc
    dddddddddddddddd
    eeeeeee
]
foo(%w[a b], %i[c d])
[1, 2].first
[1, 2].map { x }
