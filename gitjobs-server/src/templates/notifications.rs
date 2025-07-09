//! Templates for notification-related emails and messages.

use askama::Template;
use serde::{Deserialize, Serialize};

use super::jobboard::jobs::Job;

use crate::templates::{dashboard::employer::jobs::Workplace, filters};

// Emails templates.

/// Template for email verification notification.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "notifications/email_verification.html")]
pub(crate) struct EmailVerification {
    /// Verification link for the user to confirm their email address.
    pub link: String,
}

/// Template for team invitation notification.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "notifications/team_invitation.html")]
pub(crate) struct TeamInvitation {
    /// Link to invitations page.
    pub link: String,
}

// Slack templates.

/// Template for the new job published Slack notification.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "notifications/slack_job_published.md")]
pub(crate) struct JobPublished {
    /// Base URL for the job board.
    pub base_url: String,
    /// Job details.
    pub job: Job,
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use askama::Template;
    use uuid::Uuid;

    use crate::templates::{
        dashboard::employer::jobs::{JobKind, Workplace},
        jobboard::jobs::{Employer, Seniority},
        misc::Location,
    };

    use super::*;

    fn create_base_job() -> Job {
        Job {
            job_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            title: "Software Engineer".to_string(),
            employer: Employer {
                employer_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
                company: "ACME Corp".to_string(),
                description: None,
                logo_id: None,
                member: None,
                website_url: None,
            },
            kind: JobKind::FullTime,
            workplace: Workplace::OnSite,
            location: None,
            seniority: None,
            salary: None,
            salary_currency: None,
            salary_min: None,
            salary_max: None,
            salary_period: None,
            open_source: None,
            upstream_commitment: None,
            skills: None,
            description: "Job description".to_string(),
            apply_instructions: None,
            apply_url: None,
            benefits: None,
            certifications: None,
            projects: None,
            published_at: None,
            qualifications: None,
            responsibilities: None,
            tz_end: None,
            tz_start: None,
            updated_at: None,
        }
    }

    fn golden_file_path(test_name: &str) -> String {
        format!("src/templates/testdata/{test_name}.golden")
    }

    fn assert_golden_file(test_name: &str, actual: &str) {
        let golden_path = golden_file_path(test_name);

        // Check if we should regenerate golden files
        if env::var("REGENERATE_GOLDEN_FILES").is_ok() {
            fs::write(&golden_path, actual).expect("Failed to write golden file");
            eprintln!("Regenerated golden file: {golden_path}");
            return;
        }

        // Read expected content from golden file
        let expected = fs::read_to_string(&golden_path)
            .unwrap_or_else(|_| panic!("Golden file not found: {golden_path}. Run tests with REGENERATE_GOLDEN_FILES=1 to generate it."));

        assert_eq!(
            actual.trim(),
            expected.trim(),
            "Output doesn't match golden file: {golden_path}"
        );
    }

    #[test]
    fn test_minimal_job() {
        let job = create_base_job();
        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("minimal_job", &rendered);
    }

    #[test]
    fn test_job_with_location_onsite() {
        let mut job = create_base_job();
        job.location = Some(Location {
            location_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
            city: "San Francisco".to_string(),
            country: "USA".to_string(),
            state: Some("CA".to_string()),
        });
        job.workplace = Workplace::OnSite;

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_location_onsite", &rendered);
    }

    #[test]
    fn test_job_with_location_remote() {
        let mut job = create_base_job();
        job.location = Some(Location {
            location_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
            city: "San Francisco".to_string(),
            country: "USA".to_string(),
            state: Some("CA".to_string()),
        });
        job.workplace = Workplace::Remote;

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_location_remote", &rendered);
    }

    #[test]
    fn test_job_with_location_hybrid() {
        let mut job = create_base_job();
        job.location = Some(Location {
            location_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
            city: "San Francisco".to_string(),
            country: "USA".to_string(),
            state: Some("CA".to_string()),
        });
        job.workplace = Workplace::Hybrid;

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_location_hybrid", &rendered);
    }

    #[test]
    fn test_job_remote_no_location() {
        let mut job = create_base_job();
        job.workplace = Workplace::Remote;

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_remote_no_location", &rendered);
    }

    #[test]
    fn test_job_with_seniority() {
        let mut job = create_base_job();
        job.seniority = Some(Seniority::Senior);

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_seniority", &rendered);
    }

    #[test]
    fn test_job_with_fixed_salary() {
        let mut job = create_base_job();
        job.salary = Some(150_000);
        job.salary_currency = Some("USD".to_string());
        job.salary_period = Some("year".to_string());

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_fixed_salary", &rendered);
    }

    #[test]
    fn test_job_with_salary_range() {
        let mut job = create_base_job();
        job.salary_min = Some(120_000);
        job.salary_max = Some(180_000);
        job.salary_currency = Some("EUR".to_string());
        job.salary_period = Some("year".to_string());

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_salary_range", &rendered);
    }

    #[test]
    fn test_job_with_salary_min_only() {
        let mut job = create_base_job();
        job.salary_min = Some(100_000);
        job.salary_currency = Some("GBP".to_string());
        job.salary_period = Some("year".to_string());

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_salary_min_only", &rendered);
    }

    #[test]
    fn test_job_with_open_source() {
        let mut job = create_base_job();
        job.open_source = Some(80);

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_open_source", &rendered);
    }

    #[test]
    fn test_job_with_upstream_commitment() {
        let mut job = create_base_job();
        job.upstream_commitment = Some(50);

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_upstream_commitment", &rendered);
    }

    #[test]
    fn test_job_with_skills() {
        let mut job = create_base_job();
        job.skills = Some(vec![
            "rust".to_string(),
            "kubernetes".to_string(),
            "docker".to_string(),
        ]);

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_with_skills", &rendered);
    }

    #[test]
    fn test_job_full_details() {
        let mut job = create_base_job();
        job.location = Some(Location {
            location_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap(),
            city: "Berlin".to_string(),
            country: "Germany".to_string(),
            state: None,
        });
        job.workplace = Workplace::Hybrid;
        job.seniority = Some(Seniority::Lead);
        job.salary_min = Some(90_000);
        job.salary_max = Some(130_000);
        job.salary_currency = Some("EUR".to_string());
        job.salary_period = Some("year".to_string());
        job.open_source = Some(100);
        job.upstream_commitment = Some(75);
        job.skills = Some(vec![
            "golang".to_string(),
            "cloud-native".to_string(),
            "devops".to_string(),
            "leadership".to_string(),
        ]);

        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job,
        };

        let rendered = template.render().unwrap();
        assert_golden_file("job_full_details", &rendered);
    }

    #[test]
    fn test_job_different_kinds() {
        let mut job = create_base_job();

        // Test contractor
        job.kind = JobKind::Contractor;
        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job: job.clone(),
        };
        let rendered = template.render().unwrap();
        assert_golden_file("job_contractor", &rendered);

        // Test internship
        job.kind = JobKind::Internship;
        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job: job.clone(),
        };
        let rendered = template.render().unwrap();
        assert_golden_file("job_internship", &rendered);

        // Test part-time
        job.kind = JobKind::PartTime;
        let template = JobPublished {
            base_url: "https://example.com".to_string(),
            job: job.clone(),
        };
        let rendered = template.render().unwrap();
        assert_golden_file("job_part_time", &rendered);
    }
}
