create or replace function i_array_to_string(text[], text)
returns text language sql immutable as $$select array_to_string($1, $2)$$;

alter table job add column tsdoc tsvector not null
    generated always as (
        setweight(to_tsvector('simple', title), 'A') ||
        setweight(to_tsvector('simple', i_array_to_string(coalesce(skills, '{}'), ' ')), 'B') ||
        setweight(to_tsvector('simple', description), 'C')
    ) stored;

---- create above / drop below ----

alter table job drop column tsdoc;
drop function i_array_to_string(text[], text);
