foo(
    # before first
    a,
)
foo(
    # after paren
    a,
)
foo(
    a, # trailing first
    b,
)
foo(
    a,
    # between
    b,
)
foo(
    a,
    b, # trailing last
)
foo(
    a,
    b,
    # after last
)
foo(
    a,
    # after blank
)
foo(
    a,
    # two
    # after last
)
foo(
    a, # trailing
    # then own line
)
foo(
    a: 1, # trailing keyword
)
foo(
    # before keyword
    a: 1,
)
foo(
    a: 1, # first keyword
    b: 2,
)
foo(
    a: 1,
    # after last keyword
)
foo(
    a, # inline
    b,
)
foo a, # command
        b
foo(a) # after call
foo(
    # before first
    a,
) { 1 }
foo(a) do |x| # after block params
    1
end
