-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set draftJobID '00000000-0000-0000-0000-000000000303'
\set employerID '00000000-0000-0000-0000-000000000101'
\set pendingApprovalJobID '00000000-0000-0000-0000-000000000301'
\set publishedJobID '00000000-0000-0000-0000-000000000302'
\set reviewerID '00000000-0000-0000-0000-000000000201'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, moderator, name, user_id, username) values
    (decode('01', 'hex'), 'reviewer@example.com', true, 'Reviewer', :'reviewerID', 'reviewer');

insert into employer (company, description, employer_id) values
    ('Acme', 'Employer for reject_job tests', :'employerID');

insert into job (description, employer_id, job_id, kind, status, title, workplace) values
    (
        'Pending approval role',
        :'employerID',
        :'pendingApprovalJobID',
        'full-time',
        'pending-approval',
        'Platform Engineer',
        'remote'
    ),
    (
        'Published role',
        :'employerID',
        :'publishedJobID',
        'full-time',
        'published',
        'Senior Engineer',
        'remote'
    ),
    (
        'Draft role',
        :'employerID',
        :'draftJobID',
        'full-time',
        'draft',
        'Draft Engineer',
        'remote'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should mark a pending-approval job as rejected and store review metadata
select reject_job(
    :'pendingApprovalJobID'::uuid,
    :'reviewerID'::uuid,
    'Needs more detail'
);

select ok(
    exists (
        select 1
        from job
        where job_id = :'pendingApprovalJobID'::uuid
        and status = 'rejected'
        and review_notes = 'Needs more detail'
        and reviewed_at is not null
        and reviewed_by = :'reviewerID'::uuid
    ),
    'Should mark a pending-approval job as rejected and store review metadata'
);

-- Should mark a published job as rejected and store review metadata
select reject_job(
    :'publishedJobID'::uuid,
    :'reviewerID'::uuid,
    'Role was not approved for republishing'
);

select ok(
    exists (
        select 1
        from job
        where job_id = :'publishedJobID'::uuid
        and status = 'rejected'
        and review_notes = 'Role was not approved for republishing'
        and reviewed_at is not null
        and reviewed_by = :'reviewerID'::uuid
    ),
    'Should mark a published job as rejected and store review metadata'
);

-- Should not reject jobs outside the allowed statuses
select reject_job(
    :'draftJobID'::uuid,
    :'reviewerID'::uuid,
    'Should be ignored'
);

select is(
    (
        select jsonb_build_object(
            'review_notes_is_null', review_notes is null,
            'reviewed_at_is_null', reviewed_at is null,
            'reviewed_by_is_null', reviewed_by is null,
            'status', status
        )
        from job
        where job_id = :'draftJobID'::uuid
    ),
    jsonb_build_object(
        'review_notes_is_null', true,
        'reviewed_at_is_null', true,
        'reviewed_by_is_null', true,
        'status', 'draft'
    ),
    'Should not reject jobs outside the allowed statuses'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
