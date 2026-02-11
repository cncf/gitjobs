-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(8);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set jobDraftID '00000000-0000-0000-0000-000000000302'
\set jobPublishedID '00000000-0000-0000-0000-000000000301'
\set profileID '00000000-0000-0000-0000-000000000201'
\set userID '00000000-0000-0000-0000-000000000401'
\set userWithoutProfileID '00000000-0000-0000-0000-000000000402'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID', 'alice'),
    (decode('02', 'hex'), 'bob@example.com', 'Bob', :'userWithoutProfileID', 'bob');

insert into job_seeker_profile (email, job_seeker_profile_id, name, summary, user_id) values
    ('alice@example.com', :'profileID', 'Alice', 'Summary', :'userID');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for apply_to_job tests', :'employerID');

insert into job (description, employer_id, job_id, kind, status, title, workplace) values
    (
        'Published role',
        :'employerID',
        :'jobPublishedID',
        'full-time',
        'published',
        'Published Job',
        'remote'
    ),
    (
        'Draft role',
        :'employerID',
        :'jobDraftID',
        'full-time',
        'draft',
        'Draft Job',
        'remote'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should insert applications for published jobs
select is(
    apply_to_job(:'jobPublishedID'::uuid, :'userID'::uuid),
    true,
    'Should insert applications for published jobs'
);

-- Should persist the inserted application row
select is(
    (
        select jsonb_build_object(
            'application_id_is_uuid',
            application_id::text ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$',
            'cover_letter', cover_letter,
            'created_at_is_recent',
            created_at between current_timestamp - interval '1 minute' and current_timestamp,
            'job_id', job_id::text,
            'job_seeker_profile_id', job_seeker_profile_id::text,
            'updated_at', updated_at
        )
        from application
        where job_id = :'jobPublishedID'::uuid
        and job_seeker_profile_id = :'profileID'::uuid
    ),
    jsonb_build_object(
        'application_id_is_uuid', true,
        'cover_letter', null,
        'created_at_is_recent', true,
        'job_id', :'jobPublishedID',
        'job_seeker_profile_id', :'profileID',
        'updated_at', null
    ),
    'Should persist the full inserted application payload'
);

-- Should not duplicate existing applications
select is(
    apply_to_job(:'jobPublishedID'::uuid, :'userID'::uuid),
    false,
    'Should not duplicate existing applications'
);

-- Should keep a single row after duplicate attempts
select is(
    (
        select jsonb_build_object(
            'application_id_is_uuid',
            application_id::text ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$',
            'cover_letter', cover_letter,
            'created_at_is_recent',
            created_at between current_timestamp - interval '1 minute' and current_timestamp,
            'job_id', job_id::text,
            'job_seeker_profile_id', job_seeker_profile_id::text,
            'updated_at', updated_at
        )
        from application
        where job_id = :'jobPublishedID'::uuid
        and job_seeker_profile_id = :'profileID'::uuid
    ),
    jsonb_build_object(
        'application_id_is_uuid', true,
        'cover_letter', null,
        'created_at_is_recent', true,
        'job_id', :'jobPublishedID',
        'job_seeker_profile_id', :'profileID',
        'updated_at', null
    ),
    'Should keep the same full payload after duplicate attempts'
);

-- Should reject applications to non-published jobs
select is(
    apply_to_job(:'jobDraftID'::uuid, :'userID'::uuid),
    false,
    'Should reject applications to non-published jobs'
);

-- Should not insert applications for non-published jobs
select ok(
    (
        not exists (
            select 1
            from application
            where job_id = :'jobDraftID'::uuid
            and job_seeker_profile_id = :'profileID'::uuid
        )
    ),
    'Should not insert applications for non-published jobs'
);

-- Should return false when the user has no job seeker profile
select is(
    apply_to_job(:'jobPublishedID'::uuid, :'userWithoutProfileID'::uuid),
    false,
    'Should return false when the user has no job seeker profile'
);

-- Should not insert rows for users without job seeker profile
select ok(
    (
        not exists (
            select 1
            from application a
            join job_seeker_profile p using (job_seeker_profile_id)
            where a.job_id = :'jobPublishedID'::uuid
            and p.user_id = :'userWithoutProfileID'::uuid
        )
    ),
    'Should not insert rows for users without job seeker profile'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
