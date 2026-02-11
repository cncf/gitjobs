-- Returns salary fields required to refresh normalized salary values.
create or replace function dashboard_employer_get_job_salary(p_job_id uuid)
returns table(
    salary bigint,
    salary_currency text,
    salary_min bigint,
    salary_max bigint,
    salary_period text
) as $$
    select
        j.salary,
        j.salary_currency,
        j.salary_min,
        j.salary_max,
        j.salary_period
    from job j
    where j.job_id = p_job_id;
$$ language sql;
