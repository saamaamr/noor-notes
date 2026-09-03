create table if not exists public.encrypted_vaults (
    owner_id uuid primary key default auth.uid() references auth.users(id) on delete cascade,
    wrapped_vault jsonb not null check (jsonb_typeof(wrapped_vault) = 'object'),
    recovery_wrapped_vault jsonb not null check (jsonb_typeof(recovery_wrapped_vault) = 'object'),
    updated_at timestamptz not null default now()
);

alter table public.encrypted_vaults enable row level security;

create policy "owners can select encrypted vault"
    on public.encrypted_vaults for select
    to authenticated
    using (owner_id = auth.uid());

create policy "owners can insert encrypted vault"
    on public.encrypted_vaults for insert
    to authenticated
    with check (owner_id = auth.uid());

create policy "owners can update encrypted vault"
    on public.encrypted_vaults for update
    to authenticated
    using (owner_id = auth.uid())
    with check (owner_id = auth.uid());

create policy "owners can delete encrypted vault"
    on public.encrypted_vaults for delete
    to authenticated
    using (owner_id = auth.uid());

grant select, insert, update, delete on public.encrypted_vaults to authenticated;

