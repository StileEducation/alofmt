-> {}
-> {}
-> { x }
->(a) { x }
->(a) { x }
->(a) { x }
->(a, b) { x }
->(a, b = 1, *c, d:, **e, &f) { x }
->(a, (b, c)) { x }
->(*) { x }
->(**nil) { x }
->(&b) { x }
-> {}
-> { x }
-> do
    x
    y
end
-> do
    x
    y
end
->(a) { x }
->(a; b) { x }
->(a, b; c, d) { x }
-> { x }
-> { it }
-> { _1 + _2 }
-> do
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa(
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccc,
    )
end
->(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccc
) do
    x
end
->(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb; cccccccccccccccc
) do
    x
end
foo(-> { x })
foo(-> { x })
foo ->(a) { x }
foo(1, ->(a) { x })
foo.each(&->(a) { x })
scope :active, -> { where(active: true) }
scope :recent, ->(days) { where('created_at > ?', days.days.ago) }
scope :recent,
            ->(days) do
                where('created_at > ?', days.days.ago).order(created_at: :desc).limit(
                    100,
                )
            end
validates :name, presence: true, if: -> { foo? }
-> { x } # trailing
->(a) { x } # trailing
->(a) do # trailing
    x
end
-> do
    # only comment
end
-> do
    # only comment
end
-> do
    x
    # before end
end
->(
    a, # comment
    b
) do
    x
end
-> { -> { x } }
->(a) { ->(b) { a + b } }
->(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb; cccccccccccccccc, ddddddddddddddddddddddd, eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee, ffffffffffffff
) do
    x
end
