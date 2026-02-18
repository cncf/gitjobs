-- Updates employer details and refreshes its memberships.
create or replace function update_employer(
    p_employer_id uuid,
    p_employer jsonb
)
returns void as $$
begin
    -- Update employer core fields
    update employer
    set
        company = p_employer->>'company',
        description = p_employer->>'description',
        public = (p_employer->>'public')::boolean,
        location_id = ((p_employer->'location')->>'location_id')::uuid,
        logo_id = (p_employer->>'logo_id')::uuid,
        website_url = p_employer->>'website_url',
        updated_at = current_timestamp
    where employer_id = p_employer_id;

    -- Replace memberships with payload values
    delete from employer_member
    where employer_id = p_employer_id;

    insert into employer_member (employer_id, member_id)
    select
        p_employer_id,
        (member->>'member_id')::uuid
    from jsonb_array_elements(
        case
            when p_employer ? 'members' and jsonb_typeof(p_employer->'members') <> 'null' then
                p_employer->'members'
            else '[]'::jsonb
        end
    ) member
    on conflict (employer_id, member_id) do nothing;
end
$$ language plpgsql;
