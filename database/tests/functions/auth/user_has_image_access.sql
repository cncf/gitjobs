-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(6);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set creatorUserID '00000000-0000-0000-0000-000000000001'
\set employerID '00000000-0000-0000-0000-000000000101'
\set employerLogoImageID '00000000-0000-0000-0000-000000000202'
\set jobID '00000000-0000-0000-0000-000000000301'
\set jobSeekerProfileID '00000000-0000-0000-0000-000000000401'
\set jobSeekerUserID '00000000-0000-0000-0000-000000000005'
\set moderatorUserID '00000000-0000-0000-0000-000000000002'
\set nonAccessibleImageID '00000000-0000-0000-0000-000000000204'
\set ownedImageID '00000000-0000-0000-0000-000000000201'
\set profilePhotoImageID '00000000-0000-0000-0000-000000000203'
\set teamUserID '00000000-0000-0000-0000-000000000003'
\set unapprovedTeamUserID '00000000-0000-0000-0000-000000000004'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Users
insert into "user" (
    user_id,
    auth_hash,
    email,
    moderator,
    name,
    username
) values
    (:'creatorUserID', decode('01', 'hex'), 'creator@example.com', false, 'Creator User', 'creator'),
    (:'moderatorUserID', decode('02', 'hex'), 'moderator@example.com', true, 'Moderator User', 'moderator'),
    (:'teamUserID', decode('03', 'hex'), 'team@example.com', false, 'Team User', 'team-user'),
    (:'unapprovedTeamUserID', decode('04', 'hex'), 'pending@example.com', false, 'Pending User', 'pending-user'),
    (:'jobSeekerUserID', decode('05', 'hex'), 'seeker@example.com', false, 'Seeker User', 'seeker-user');

-- Images
insert into image (image_id, created_by) values
    (:'ownedImageID', :'creatorUserID'),
    (:'employerLogoImageID', null),
    (:'profilePhotoImageID', null),
    (:'nonAccessibleImageID', :'creatorUserID');

-- Employer and team
insert into employer (employer_id, company, description, logo_id)
values (:'employerID', 'Test Employer', 'Employer used in access tests', :'employerLogoImageID');

insert into employer_team (approved, employer_id, user_id) values
    (true, :'employerID', :'teamUserID'),
    (false, :'employerID', :'unapprovedTeamUserID');

-- Job and application profile
insert into job (
    job_id,
    employer_id,
    kind,
    status,
    title,
    workplace,
    description
) values (
    :'jobID',
    :'employerID',
    'full-time',
    'published',
    'Platform Engineer',
    'remote',
    'Role used in image access tests'
);

insert into job_seeker_profile (
    job_seeker_profile_id,
    user_id,
    photo_id,
    email,
    name,
    summary
) values (
    :'jobSeekerProfileID',
    :'jobSeekerUserID',
    :'profilePhotoImageID',
    'seeker@example.com',
    'Seeker User',
    'Test profile for image access checks'
);

insert into application (job_id, job_seeker_profile_id)
values (:'jobID', :'jobSeekerProfileID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should allow moderators to access any image
select ok(
    auth_user_has_image_access(
        :'moderatorUserID'::uuid,
        'ffffffff-ffff-ffff-ffff-ffffffffffff'::uuid
    ),
    'Should allow moderators to access any image'
);

-- Should allow users to access images they created
select ok(
    auth_user_has_image_access(:'creatorUserID'::uuid, :'ownedImageID'::uuid),
    'Should allow users to access images they created'
);

-- Should allow approved employer members to access candidate profile photos
select ok(
    auth_user_has_image_access(:'teamUserID'::uuid, :'profilePhotoImageID'::uuid),
    'Should allow approved employer members to access candidate profile photos'
);

-- Should allow approved employer members to access employer logos
select ok(
    auth_user_has_image_access(:'teamUserID'::uuid, :'employerLogoImageID'::uuid),
    'Should allow approved employer members to access employer logos'
);

-- Should deny access to unapproved employer members for employer logos
select ok(
    not auth_user_has_image_access(:'unapprovedTeamUserID'::uuid, :'employerLogoImageID'::uuid),
    'Should deny access to unapproved employer members for employer logos'
);

-- Should deny access when no access rule matches
select ok(
    not auth_user_has_image_access(:'jobSeekerUserID'::uuid, :'nonAccessibleImageID'::uuid),
    'Should deny access when no access rule matches'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
