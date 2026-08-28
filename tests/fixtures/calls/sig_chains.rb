sig do
    params(source_record_id: String)
        .returns(Atlas::Services::Catalog::PublishedRecord::BatchGetResponse)
        .checked(:never)
end
sig do
    params(source_record_id: String).returns(
        Atlas::Services::Catalog::PublishedRecord::BatchGetResponse,
    )
end
sig do
    foo
        .bbbbbbbbbbbbbbbbbbbbbbbbbbb(1_111_111_111_111_111_111_111)
        .cccccccccccccccccccccccccccc(2_222_222_222_222_222)
end
sig do
    foo
        .bbbbbbbbbbbbbbbbbbbbbbbbbbb(1_111_111_111_111_111_111_111)
        .cccccccccccccccccccccccccccc(2_222_222_222_222_222)
end
sig(:final) do
    foo
        .bbbbbbbbbbbbbbbbbbbbbbbbbbb(1_111_111_111_111_111_111_111)
        .cccccccccccccccccccccccccccc(2_222_222_222_222_222)
end
T::Sig.sig do
    foo
        .bbbbbbbbbbbbbbbbbbbbbbbbbbb(1_111_111_111_111_111_111_111)
        .cccccccccccccccccccccccccccc(2_222_222_222_222_222)
end
sig do
    x =
        foo.bbbbbbbbbbbbbbbbbbbbbbbbbbb(
            1_111_111_111_111_111_111_111,
        ).cccccccccccccccccccccccccccc(2_222_222_222_222_222)
    bar(
        foo.bbbbbbbbbbbbbbbbbbbbbbbbbbb(
            1_111_111_111_111_111_111_111,
        ).cccccccccccccccccccccccccccc(2_222_222_222_222_222),
    )
    override
        .params(aaaaaaaaaaaaaaaaa: 1, bbbbbbbbbbbbbbbbbbbbbbbbbbbb: 2)
        .returns(Looooooooooooooooooooooooooooooong)
    params(a: Integer).void
end
sigx do
    foo.bbbbbbbbbbbbbbbbbbbbbbbbbbb(
        1_111_111_111_111_111_111_111,
    ).cccccccccccccccccccccccccccc(2_222_222_222_222_222)
end
foo do
    foo.bbbbbbbbbbbbbbbbbbbbbbbbbbb(
        1_111_111_111_111_111_111_111,
    ).cccccccccccccccccccccccccccc(2_222_222_222_222_222)
    foo
        .bbbbbbbbbbbbbbbbbbbbbbbbbbb(1_111_111_111_111_111_111_111)
        .cccccccccccccccccccccccccccc(2_222_222_222_222_222)
        .dd
end
