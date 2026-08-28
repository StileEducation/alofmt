a
    .b
    .c(1) # trailing
    .d # trailing d
a.b(1).c { |x| x }.d # tail
a # c1
    .b # c2
    .c
a.b(1) # c1
    .c
warehouse[:import_errors]
    .where(filename: object_uris) # Find imports for this object
    .order(
        ::Sequel.desc(:query),
    ) # Get the latest import on the assumption that ours will be last
    .first
ranked
    .select do |entry|
        corpus.include?(entry.record_id)
    end # One entry per record, even if a batch ranked one of them twice
    .group_by(&:record_id)
    .map { |record_id, scores| scores }
db[:workspaceMember].insert_conflict(target: %i[userId workspaceId]) # NOOP on conflict
    .multi_insert(
    new_member_requests.map do |member_req|
        { workspaceId: member_req.workspace_id, userId: member_req.user_id }
    end,
)
synced_query
    # Make sure to select only the columns we use
    .select_columns(REMOTE_ASSET_ROW_COLUMNS)
    .all
    .filter_map { |row| row }
x =
    a
        .b(
            a,
            # trailing own-line
        )
        .c
        .d
