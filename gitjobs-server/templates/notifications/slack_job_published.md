New job published!

{# Job title and company -#}
*{{ job.title }}* at *{{ job.employer.company }}*
{#- End job title and company #}

{# Job type -#}
• _Job type:_ *{{ &job.kind.to_string()|unnormalize|capitalize }}*
{# End job type -#}
{# Location -#}
• _Location:_{{ " " }}
{%- if let Some(location) = job.location -%}
  *{{ location.city }}, {{ location.country }}*{{ " " }}
  {%- if job.workplace == Workplace::Remote -%}
  (remote)
  {%- else if job.workplace == Workplace::Hybrid -%}
  (hybrid)
  {%- endif -%}
{%- else -%}
  {%- if job.workplace == Workplace::Remote -%}
  *Remote*
  {%- else -%}
  *Not provided*
  {%- endif -%}
{%- endif %}
{# End location -#}
{# Seniority -#}
{% if let Some(seniority) = job.seniority -%}
• _Seniority:_ *{{ &seniority.to_string()|unnormalize|capitalize }}*
{% endif -%}
{# End seniority -#}
{# Salary -#}
• _Salary:_{{ " " }}*
{%- if let Some(salary) = job.salary -%}
  {%- if let Some(salary_currency) = job.salary_currency -%}
    {{ salary_currency }}{{ " " }}
  {%- endif -%}
  {{ salary|humanize_salary }}{{ " " }}
  {%- if let Some(salary_period) = job.salary_period -%}
    / {{ salary_period }}
  {%- endif -%}*
{%- else if let Some(salary_min) = job.salary_min -%}
  {%- if let Some(salary_currency) = job.salary_currency -%}
    {{ salary_currency }}{{ " " }}
  {%- endif -%}
  {{ salary_min|humanize_salary }}{{ " " }}
  {%- if let Some(salary_max) = job.salary_max -%}
    - {{ salary_max|humanize_salary }}{{ " " }}
  {%- endif -%}
  {%- if let Some(salary_period) = job.salary_period -%}
    / {{ salary_period }}
  {%- endif -%}*
{%- else -%}
  Not provided*
{%- endif %}
{# End salary -#}
{# Open source -#}
{% if let Some(open_source) = job.open_source -%}
• _Time working on open source:_ *{{ open_source }}%*
{% endif -%}
{# End open source -#}
{# Upstream commitment -#}
{% if let Some(upstream_commitment) = job.upstream_commitment -%}
• _Time working on upstream projects:_ *{{ upstream_commitment }}%*
{% endif -%}
{# End upstream commitment -#}
{# Skills -#}
{% if let Some(skills) = job.skills -%}
• _Required skills:_ {% for skill in skills.iter().take(5) -%}*` {{ skill|unnormalize|capitalize }} `*  {% endfor %}
{% endif -%}
{# End skills -#}
{{ " " }}
For more details please see: {{ base_url }}/?job_id={{ job.job_id }}
