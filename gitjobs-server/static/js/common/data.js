/**
 * Returns a list of available job benefits.
 * Used for multiselect and filter components.
 * @returns {string[]} Array of benefit identifiers
 */
export const getBenefits = () => {
  return [
    "401k",
    "flexible-hours",
    "remote-friendly",
    "health-insurance",
    "paid-time-off",
    "4-day-workweek",
    "company-retreats",
    "home-office-budget",
    "learning-budget",
    "mental-wellness-bugdet",
    "equity-compensation",
    "no-whiteboard-interview",
  ];
};

/**
 * Returns a list of available technical skills.
 * Used for multiselect and filter components.
 * @returns {string[]} Array of skill identifiers
 */
export const getSkills = () => {
  return [
    "kubernetes",
    "docker",
    "aws",
    "gcp",
    "azure",
    "terraform",
    "linux",
    "helm",
    "prometheus",
    "python",
    "golang",
    "rust",
    "jenkins",
    "java",
    "git",
    "devops",
    "ansible",
    "ci/cd",
    "sre",
    "security",
    "containers",
    "oci",
    "c++",
    "serverless",
    "automation",
    "microservices",
    "service-mesh",
  ];
};
