x = # value below
    1
x += # operator assign
    1
x = # call value
    foo(1)
foo.bar = # attribute write
    1
def foo # no params
    1
end
def foo # empty body
end
def self.foo # receiver
    1
end
def foo() # empty parens
    1
end
def foo( # after paren
    a
)
    1
end
def foo(a) # after params
    1
end
def foo(
    # before first
    a,
    b # trailing last
)
    1
end
def foo(
    a, # trailing first
    b
)
    1
end
x = ->(a) do # lambda params
    1
end
