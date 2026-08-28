a, b = 1, 2
a, b = b, a
a, b = foo
a, b = *foo
a, *b = foo
*a, b = foo
a, = foo
a, * = foo
*, a = foo
a, (b, c) = foo
a, (b, *c), d = foo
a, (b,) = foo
(a, b), c = foo
a, b = foo
a, b = [1, 2]
a.b, c[1] = 1, 2
a[1], a.b, A::B, ::C, *d = foo
A::B, ::C, $d, @e, @@f = 1, 2, 3, 4, 5
foo.bar, baz = 1, 2
a, b =
    foo(
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    )
aaaaaaaaaaaaaaaaaaaaaaaaaa,
bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
ccccccccccccccccccccccccccc =
    1,
    2,
    3_333_333
aaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb =
    ccccccccccccccccccccccccccc,
    ddddddddddddddd
a, b =
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    b
a, b =
    [
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        b,
    ]
a, b =
    {
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:
            b,
    }
a, b =
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.map do
        c
    end
a, b =
    *aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa[bbbbbbbbbbbbbbbbbbbbbbbbbb],
ccccccccccccccccccccccccccc =
    1,
    2
aaaaaaaaaaaaaaaaaaaaaaa,
(bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccccccccccccccccccc),
ddddddddddd =
    foo
a,
(
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    ccccccccccccccccccccccccccccccccccccccccccccccc
) =
    foo
a, b = c, d = 1, 2
a, b = c = 1
c = a, b = 1, 2
a,
b = # comment
    1,
    2
a, b = foo
(a, b), c = foo
