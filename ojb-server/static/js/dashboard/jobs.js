import { processNewHtmxUrl } from "../common/common.js";

// Trigger action on the form provided.
export const triggerActionOnForm = (formId, action) => {
  const form = document.getElementById(formId);
  if (form) {
    htmx.trigger(form, action);
  }
};

export const setupActionsButton = () => {
  const ALL_BUTTONS_ACTIONS = ['publish', 'archive', 'delete', 'update'];
  const METHODS = {
    publish: 'PUT',
    archive: 'PUT',
    delete: 'DELETE',
    update: 'GET'
  }

  const noActions = document.getElementById('placeholder-actions');

  const selectedJob = document.querySelector('#jobs-list input[type="radio"]:checked');
  if (selectedJob) {
    // Hide placeholder actions when a job is selected
    noActions.classList.add('hidden');

    const visibleButtons = [];

    // Get job info
    const jobInfo = JSON.parse(selectedJob.dataset.job);

    // Add update button to list of visible buttons
    visibleButtons.push('update');

    // Add delete button to list of visible buttons
    visibleButtons.push('delete');

    if (jobInfo.status !== 'published') {
      // Add publish button to list of visible buttons if job is not published
      visibleButtons.push('publish');
    } else {
      // Add archive button to list of visible buttons if job is published
      visibleButtons.push('archive');
    }

    ALL_BUTTONS_ACTIONS.forEach((action) => {
      const button = document.getElementById(`${action}-button`);
      if (visibleButtons.includes(action)) {
        const method = METHODS[action].toLowerCase();

        // Update button URL with job ID
        processNewHtmxUrl(`${action}-button`, method, jobInfo.job_id);

        // Add event listener to update jobs list after request
        // except for update action
        if (action !== 'update') {
          button.addEventListener('htmx:afterRequest', () => {
            updateJobsList();
          });
        } else {
          button.addEventListener('htmx:afterRequest', () => {
            history.pushState({}, "Jobs list", '/dashboard?tab=jobs');
          });
        }

        button.parentElement.classList.remove('hidden');
      } else {
        button.parentElement.classList.add('hidden');
      }
    });

  } else {
    // Display placeholder actions when no job is selected
    noActions.classList.remove('hidden');
  }
}

export const updateJobsList = () => {
  const refreshButton = document.getElementById('refresh-table');
  htmx.trigger(refreshButton, 'click');
}

