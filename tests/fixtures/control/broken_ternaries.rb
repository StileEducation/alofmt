aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
    c ? d : e :
    bbbbbbbbbbbbbbbbbbbb
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
    x = 1 :
    bbbbbbbbbbbbbbbbbbbb
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
    -> { c } :
    bbbbbbbbbbbbbbbbbbbb
foo(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
        bbbbbbbbbbbbbbbbbbbb :
        c ? d : e,
)
foo(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
        bbbbbbbbbbbbbbbbbbbb :
        c ? d : e,
    1,
)
(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
        bbbbbbbbbbbbbbbbbbbb :
        c ? d : e
)
[
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
        bbbbbbbbbbbbbbbbbbbb :
        c ? d : e,
]
x =
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
        bbbbbbbbbbbbbbbbbbbb :
        -> { c }
x ||=
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
        bbbbbbbbbbbbbbbbbbbb :
        -> { c }
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
    foo(
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        ccccccccccc,
    ) :
    c ? d : e
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa &&
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ?
    bbbbbbbbbbbbbbbbbbbb :
    c ? d : e
a ? b : c ? d : e
a ? b ? c : d : e
a ? x = 1 : b
a ? -> { c } : b
a ? b : c.d = 1
a ? b : (c ? d : e)
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
    bbbbbbbbbbbbbbbbbbbb :
    c ? d : e ? f : g
if z
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
        bbbbbbbbbbbbbbbbbbbb :
        c ? d : e
end
return(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
        bbbbbbbbbbbbbbbbbbbb :
        c ? d : e
)
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
    bbbbbbbbbbbbbbbbbbbb :
    c ? d : e # cc
def foo
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ?
        bbbbbbbbbbbbbbbbbbbb :
        c ? d : e
end
