-- Rejects a job and updates review metadata.
create or replace function reject_job(
    p_job_id uuid,
    p_reviewer uuid,
    p_review_notes text
)
returns void as $$
    update job
    set
        status = 'rejected',
        review_notes = p_review_notes,
        reviewed_at = current_timestamp,
        reviewed_by = p_reviewer
    where job_id = p_job_id;
$$ language sql;
