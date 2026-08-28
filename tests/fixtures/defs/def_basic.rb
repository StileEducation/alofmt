def foo(a, b)
    x
end
def foo()
end
def foo
end
def foo
end
def foo(a, b = 1, *c, d, e:, f: 2, **g, &h)
    x
end
def self.foo(a)
    x
end
def obj.foo
end
def Foo.bar
end
def self.foo
end
def -(other)
end
def `(x)
end
def foo?() = 1
def foo!(a)
end
def foo=(v)
end
def []=(k, v)
end
def <=>(other)
    a <=> other.a
end
def foo
    x

    y
end

def foo
    x
end
def foo(a) # trailing
end
def foo(a)
end # trailing end
def foo(a) # trailing
end # trailing end
def foo(a, b) # trailing
    x
end
def foo
    # own line before end
end
def foo
    x
    # own line before end
end
# leading comment
def foo
    x
end
# comment between defs

def bar
    x
end
private def priv(a)
    a
end
private def priv = 1
def foo(a, b = 1, *c, d, e:, f: 2, **g, &h) # trailing
    x
end
