create table if not exists public.encrypted_note_revisions (
    owner_id uuid not null default auth.uid() references auth.users(id) on delete cascade,
    note_id uuid not null,
    revision bigint not null check (revision >= 0),
    ciphertext text not null,
    nonce text not null,
    updated_at timestamptz not null,
    deleted_at timestamptz,
    primary key (owner_id, note_id, revision)
);

create index if not exists encrypted_note_revisions_owner_updated_idx
    on public.encrypted_note_revisions (owner_id, updated_at);

alter table public.encrypted_note_revisions enable row level security;

create policy "owners can select encrypted revisions"
    on public.encrypted_note_revisions for select
    to authenticated
    using (owner_id = auth.uid());

create policy "owners can insert encrypted revisions"
    on public.encrypted_note_revisions for insert
    to authenticated
    with check (owner_id = auth.uid());

create policy "owners can update encrypted revisions"
    on public.encrypted_note_revisions for update
    to authenticated
    using (owner_id = auth.uid())
    with check (owner_id = auth.uid());

create policy "owners can delete encrypted revisions"
    on public.encrypted_note_revisions for delete
    to authenticated
    using (owner_id = auth.uid());

grant select, insert, update, delete on public.encrypted_note_revisions to authenticated;
