create table certification (
    certification_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> ''),
    provider text not null check (provider <> ''),
    short_name text not null unique check (short_name <> ''),

    description text check (description <> ''),
    logo_url text check (logo_url <> ''),
    url text check (url <> '')
);

insert into certification (name, provider, short_name, description, url, logo_url) values
('Certified Kubernetes Administrator', 'CNCF', 'CKA', 'Performance-based exam where candidates interact with the command line to solve real-world challenges', 'https://www.cncf.io/training/certification/cka/', 'https://www.cncf.io/wp-content/uploads/2021/09/kubernetes-cka-color.svg'),
('Certified Kubernetes Application Developer', 'CNCF', 'CKAD', 'Proves you can design, build, and manage cloud-native applications on Kubernetes', 'https://www.cncf.io/training/certification/ckad/', 'https://www.cncf.io/wp-content/uploads/2021/09/kubernetes-ckad-color.svg'),
('Certified Kubernetes Security Specialist', 'CNCF', 'CKS', 'Covers best practices for securing container-based applications and Kubernetes platforms', 'https://www.cncf.io/training/certification/cks/', 'https://www.cncf.io/wp-content/uploads/2020/11/kubernetes-security-specialist-logo.svg'),
('Kubernetes and Cloud Native Associate', 'CNCF', 'KCNA', 'Entry-level credential focusing on Kubernetes and broader cloud native ecosystem', 'https://www.cncf.io/training/certification/kcna/', 'https://www.cncf.io/wp-content/uploads/2021/09/kcna_color.svg'),
('Kubernetes and Cloud Security Associate', 'CNCF', 'KCSA', 'Validates skills in evaluating Kubernetes cluster security configurations and compliance', 'https://www.cncf.io/training/certification/kcsa/', 'https://www.cncf.io/wp-content/uploads/2024/03/kubernetes-kcsa-color.svg'),
('Prometheus Certified Associate', 'CNCF', 'PCA', 'Entry-level certification demonstrating foundational knowledge of observability and Prometheus monitoring', 'https://www.cncf.io/training/certification/pca/', 'https://www.cncf.io/wp-content/uploads/2023/11/PCA-Prometheus-Certified-Associate-logo-color.svg'),
('Istio Certified Associate', 'CNCF', 'ICA', 'Pre-professional certification demonstrating foundational knowledge of Istio principles, terminology, and best practices', 'https://www.cncf.io/training/certification/ica/', 'https://www.cncf.io/wp-content/uploads/2024/03/ica-icon-color.svg'),
('Cilium Certified Associate', 'CNCF', 'CCA', 'Entry-level certification designed for platform or cloud engineers interested in networking, security, and observability', 'https://www.cncf.io/training/certification/cca/', 'https://www.cncf.io/wp-content/uploads/2024/03/cca-icon-color.svg'),
('Certified Argo Project Associate', 'CNCF', 'CAPA', 'Designed for engineers, data scientists, and others interested in demonstrating their understanding of the Argo Project ecosystem', 'https://www.cncf.io/training/certification/capa/', 'https://www.cncf.io/wp-content/uploads/2024/03/capa-icon-color.svg'),
('GitOps Certified Associate', 'CNCF', 'CGOA', 'For DevOps engineers and team members, platform and software engineers, CI/CD practitioners interested in GitOps', 'https://www.cncf.io/training/certification/cgoa/', 'https://www.cncf.io/wp-content/uploads/2024/03/gitops_associate.svg'),
('Certified Backstage Associate', 'CNCF', 'CBA', 'Proves you have the skills & the mindset to work with Backstage to advance your career, your team & your organization', 'https://www.cncf.io/training/certification/cba/', 'https://www.cncf.io/wp-content/uploads/2024/11/lft_badge_backstage_associate1.svg'),
('OpenTelemetry Certified Associate', 'CNCF', 'OTCA', 'Prove your expertise in OpenTelemetry – the industry standard for tracing, metrics & logs', 'https://www.cncf.io/training/certification/otca/', 'https://www.cncf.io/wp-content/uploads/2024/11/lft_badge_opentelemetry_associate1.svg'),
('Kyverno Certified Associate', 'CNCF', 'KCA', 'Position yourself as an expert in managing and securing Kubernetes environments. Kyverno expertise shows you understand the advanced aspects of cloud management and security', 'https://www.cncf.io/training/certification/kca/', 'https://www.cncf.io/wp-content/uploads/2024/11/lft_badge_kyverno_associate1.svg');

create table job_certification (
    job_id uuid not null references job on delete cascade,
    certification_id uuid not null references certification on delete restrict,

    primary key (job_id, certification_id)
);

create index job_certification_job_id_idx on job_certification (job_id);
create index job_certification_certification_id_idx on job_certification (certification_id);

---- create above / drop below ----

drop table job_certification;
drop table certification;
