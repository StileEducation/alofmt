case [id, ids]
in [NilClass, NilClass]
    raise ArgumentError.new('Must supply one of id or ids')
in [_, NilClass]
    [T.must(id)]
in [NilClass, _]
    T.must(ids)
else
    raise ArgumentError.new('Must supply one of id or ids, not both')
end
case foo
in 1
    bar
in 2 | 3
    baz
in Integer => n if n.positive?
    qux(n)
in String unless bar?
    quux
in nil
in 3
    z
in 4
else
end
case foo
in 1
    x
end
case foo.bar(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
)
in 1
    x
end
