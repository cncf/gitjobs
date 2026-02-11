-- Deletes a team member while preserving at least one approved member.
create or replace function delete_team_member(
    p_employer_id uuid,
    p_user_id uuid
)
returns void as $$
declare
    v_approved boolean;
begin
    -- Check the target membership state
    select approved
    into v_approved
    from employer_team
    where employer_id = p_employer_id
    and user_id = p_user_id;

    if not found then
        raise exception 'team member not found';
    end if;

    -- Approved members can only be removed if another approved member remains
    if v_approved then
        delete from employer_team
        where employer_id = p_employer_id
        and user_id = p_user_id
        and (
            select count(*)
            from employer_team
            where employer_id = p_employer_id
            and approved = true
        ) > 1;

        if not found then
            raise exception 'cannot remove last approved team member';
        end if;
    else
        -- Pending invitations can be deleted directly
        delete from employer_team
        where employer_id = p_employer_id
        and user_id = p_user_id;
    end if;
end
$$ language plpgsql;
