-- Checks whether the user belongs to the employer team as an approved member.
create or replace function user_owns_employer(p_user_id uuid, p_employer_id uuid)
returns boolean as $$
    select exists (
        select 1
        from employer_team
        where user_id = p_user_id
        and employer_id = p_employer_id
        and approved = true
    );
$$ language sql;
