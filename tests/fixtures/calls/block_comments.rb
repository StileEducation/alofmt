foo do
    # comment
end
foo do
    # comment
end
foo do |x|
    # comment
end
foo do
    # a
    # b
end
foo do # c
end
foo do |x| # c
end
foo.bar do # c
end
foo
    .bar # c
    .baz
    .qux
    .quux
foo
    # c
    .bar
    .baz
    .qux
    .quux
foo
    .bar(1) # c
    .baz { x }
    .qux
