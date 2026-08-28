foo((b if a))
foo (b if a)
foo((b if a))
foo((a ? b : c))
foo.bar((a ? b : c))
foo(bar((a ? b : c)))
[(a ? b : c)]
{ a: (a ? b : c) }
{ a: (b if c) }
!(
    if a
        b
    else
        c
    end
)
!(b if a)
!(a ? b : c)
a && (b ? c : d)
foo { b if a }
(a ? b : c).foo
(b if a).foo
(b if a).foo
(
    if a
        b
    else
        c
    end
).foo
[1].each { b if a }
foo((b if a), c)
begin
    b if a
end
b if a while x
(b if a) and c
b if a if c
b if a ? c : d
puts((b if a).to_s)
(
    b if a
    c
)
foo (b if a) ? c : d
foo(
    (
        if bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
            ccccccccccccccccccccc
        else
            dddddddddddddddddddddd
        end
    ),
    e,
)
foo(
    if aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        ccccccccccccccccccccc
    else
        dddddddddddddddddddddd
    end,
    e,
)
foo(
    (
        if aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
            bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
        end
    ),
)
[
    (
        if aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
            b
        end
    ),
]
