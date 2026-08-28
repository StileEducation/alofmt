def f
    return :a => b, c: d
    return :a => b
    return foo, :a => b
    return 'a' => b
    return { a: b }
    yield a: b
    super a: b
end
