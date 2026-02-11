-- Adds a new employer with memberships and an initial approved team member.
create or replace function dashboard_employer_add_employer(
    p_user_id uuid,
    p_employer jsonb
)
returns uuid as $$
declare
    v_employer_id uuid;
begin
    -- Insert employer data from payload
    insert into employer (
        company,
        description,
        public,
        location_id,
        logo_id,
        website_url
    ) values (
        p_employer->>'company',
        p_employer->>'description',
        (p_employer->>'public')::boolean,
        ((p_employer->'location')->>'location_id')::uuid,
        (p_employer->>'logo_id')::uuid,
        p_employer->>'website_url'
    )
    returning employer_id into v_employer_id;

    -- Insert employer memberships from payload
    insert into employer_member (employer_id, member_id)
    select
        v_employer_id,
        (member->>'member_id')::uuid
    from jsonb_array_elements(
        case
            when p_employer ? 'members' and jsonb_typeof(p_employer->'members') <> 'null' then
                p_employer->'members'
            else '[]'::jsonb
        end
    ) member
    on conflict (employer_id, member_id) do nothing;

    -- Add creator as an approved team member
    insert into employer_team (employer_id, user_id, approved)
    values (v_employer_id, p_user_id, true);

    return v_employer_id;
end
$$ language plpgsql;
