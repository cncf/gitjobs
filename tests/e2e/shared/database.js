import { execFile } from "node:child_process";
import { promisify } from "node:util";

// Cleanup follows libpq variables and the repository's local database task settings.
const databaseEnvironment = {
  ...process.env,
  PGDATABASE: process.env.PGDATABASE || process.env.GITJOBS_DB_NAME || "gitjobs",
  PGHOST: process.env.PGHOST || process.env.GITJOBS_DB_HOST || "localhost",
  PGPASSWORD: process.env.PGPASSWORD ?? "password",
  PGPORT: process.env.PGPORT || process.env.GITJOBS_DB_PORT || "5432",
  PGUSER: process.env.PGUSER || process.env.GITJOBS_DB_USER || "postgres",
};
const execFileAsync = promisify(execFile);
const JOB_TITLE_PREFIX_PATTERN = /^[A-Za-z0-9 -]+$/;
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

/**
 * Removes E2E jobs whose title starts with a controlled test prefix.
 * @param {string} titlePrefix - Prefix reserved for E2E-created jobs.
 * @returns {Promise<void>}
 */
export const deleteJobsByTitlePrefix = async (titlePrefix) => {
  validateJobTitlePrefix(titlePrefix);
  await runSql(`delete from job where title like '${titlePrefix}%'`);
};

/**
 * Removes mutable applications created by a seeded E2E user.
 * @param {string} userId - Seeded user identifier.
 * @returns {Promise<void>}
 */
export const resetJobApplications = async (userId) => {
  validateUuid(userId);
  await runSql(
    `delete from application where job_seeker_profile_id in (` +
      `select job_seeker_profile_id from job_seeker_profile ` +
      `where user_id = '${userId}')`,
  );
};

/**
 * Removes a seeded user's mutable job-seeker state after a lifecycle test.
 * @param {string} userId - Seeded user identifier.
 * @returns {Promise<void>}
 */
export const resetJobSeekerProfile = async (userId) => {
  validateUuid(userId);
  await runSql(`delete from job_seeker_profile where user_id = '${userId}'`);
};

/**
 * Restores a seeded moderation job to its pending state.
 * @param {string} jobId - Seeded job identifier.
 * @returns {Promise<void>}
 */
export const resetModeratedJob = async (jobId) => {
  validateUuid(jobId);
  await runSql(
    `update job set status = 'pending-approval', first_published_at = null, ` +
      `published_at = null, review_notes = null, reviewed_at = null, ` +
      `reviewed_by = null where job_id = '${jobId}'`,
  );
};

/**
 * Executes a cleanup statement through the configured PostgreSQL client.
 * @param {string} query - Cleanup statement.
 * @returns {Promise<void>}
 */
const runSql = async (query) => {
  await execFileAsync("psql", ["-v", "ON_ERROR_STOP=1", "-c", query], {
    env: databaseEnvironment,
  });
};

/**
 * Validates a controlled title prefix before interpolating it into cleanup SQL.
 * @param {string} value - Title prefix to validate.
 * @returns {void}
 */
const validateJobTitlePrefix = (value) => {
  if (!JOB_TITLE_PREFIX_PATTERN.test(value)) {
    throw new Error(`Invalid E2E job title prefix: ${value}`);
  }
};

/**
 * Validates an identifier before interpolating it into cleanup SQL.
 * @param {string} value - Identifier to validate.
 * @returns {void}
 */
const validateUuid = (value) => {
  if (!UUID_PATTERN.test(value)) {
    throw new Error(`Invalid E2E UUID: ${value}`);
  }
};
