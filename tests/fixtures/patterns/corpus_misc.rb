case [method, m]
in ['POST', ::MatchData]
    create_build(m[:org], m[:pipeline], body)
in ['GET', ::MatchData]
    list_or_get(m)
in ['PUT', ::MatchData] if cancel?(m)
    cancel_build(m[:org], m[:pipeline], m[:n].to_i)
else
    not_implemented(method, path)
end
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
