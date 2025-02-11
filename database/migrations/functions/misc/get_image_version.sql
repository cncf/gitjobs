-- Returns an image version. We'll try first to get the version of the size
-- requested. If it doesn't exist, we'll return the svg version (if available).
create or replace function get_image_version(p_image_id uuid, p_version text)
returns table(data bytea, format text) as $$
begin
    return query select iv.data, 'png' as format from image_version iv
    where image_id = p_image_id and version = p_version;

    if found then return; end if;

    return query select iv.data, 'svg' as format from image_version iv
    where image_id = p_image_id and version = 'svg';
end
$$ language plpgsql;
