-- Check if the user has access to the image provided.
create or replace function user_has_image_access(p_user_id uuid, p_image_id uuid)
returns boolean as $$
begin
    -- User created the image (i.e. user photo or employer logo)
    perform from image
    where image_id = p_image_id
    and created_by = p_user_id;
    if found then return true; end if;

    -- User belongs to the employer team (i.e employer logo)
    perform from employer e
    join employer_team et using (employer_id)
    where e.logo_id = p_image_id
    and et.user_id = p_user_id;
    return found;
end
$$ language plpgsql;
