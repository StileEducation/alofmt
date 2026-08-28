x = # bracketed values move down after a header comment
    [1]
x = # c
    { a: 1 }
x = # c
    foo
x ||= [1] # op-assign keeps the value inline
x += [1] # c
x ||= # c
    foo
@x ||= { a: 1 } # c
x.y = # c
    [1]
x.y ||= [1] # c
A::B = # c
    [1]
a,
b = # c
    [1, 2]
