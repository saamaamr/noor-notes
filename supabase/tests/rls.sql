begin;
select plan(4);

select policies_are(
    'public',
    'encrypted_note_revisions',
    array[
        'owners can select encrypted revisions',
        'owners can insert encrypted revisions',
        'owners can update encrypted revisions',
        'owners can delete encrypted revisions'
    ]
);
select row_security_active('public.encrypted_note_revisions');
select has_index('public', 'encrypted_note_revisions', 'encrypted_note_revisions_owner_updated_idx');
select col_is_pk('public', 'encrypted_note_revisions', array['owner_id', 'note_id', 'revision']);

select * from finish();
rollback;
