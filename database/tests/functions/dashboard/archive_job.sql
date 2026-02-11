-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set draftJobID '00000000-0000-0000-0000-000000000301'
\set employerID '00000000-0000-0000-0000-000000000101'
\set pendingApprovalJobID '00000000-0000-0000-0000-000000000303'
\set publishedJobID '00000000-0000-0000-0000-000000000302'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for archive_job tests', :'employerID');

insert into job (job_id, employer_id, kind, status, title, workplace, description) values
    (
        :'draftJobID',
        :'employerID',
        'full-time',
        'draft',
        'Draft Role',
        'remote',
        'Draft description'
    ),
    (
        :'publishedJobID',
        :'employerID',
        'full-time',
        'published',
        'Published Role',
        'remote',
        'Published description'
    ),
    (
        :'pendingApprovalJobID',
        :'employerID',
        'full-time',
        'pending-approval',
        'Pending Approval Role',
        'remote',
        'Pending approval description'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should archive a published job and set archived_at
select archive_job(:'publishedJobID'::uuid);

select ok(
    exists (
        select 1
        from job
        where job_id = :'publishedJobID'::uuid
        and status = 'archived'
        and archived_at is not null
        and updated_at is not null
    ),
    'Should archive a published job and set archived_at'
);

-- Should archive a pending-approval job and set archived_at
select archive_job(:'pendingApprovalJobID'::uuid);

select ok(
    exists (
        select 1
        from job
        where job_id = :'pendingApprovalJobID'::uuid
        and status = 'archived'
        and archived_at is not null
        and updated_at is not null
    ),
    'Should archive a pending-approval job and set archived_at'
);

-- Should not archive jobs outside the allowed statuses
select archive_job(:'draftJobID'::uuid);

select is(
    (
        select jsonb_build_object(
            'archived_at_is_null', archived_at is null,
            'status', status,
            'updated_at_is_null', updated_at is null
        )
        from job
        where job_id = :'draftJobID'::uuid
    ),
    jsonb_build_object(
        'archived_at_is_null', true,
        'status', 'draft',
        'updated_at_is_null', true
    ),
    'Should not archive jobs outside the allowed statuses'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
