-- Checks whether the image is publicly visible.
create or replace function auth_is_image_public(p_image_id uuid)
returns boolean as $$
    select exists (
        select 1
        from employer e
        join job j using (employer_id)
        where e.logo_id = p_image_id
        and j.first_published_at is not null
    );
$$ language sql;
