import { LitElement, html, repeat } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";

export class ProjectsSection extends LitElement {
  static properties = {
    projects: { type: Array },
  };

  constructor() {
    super();
    this.projects = [];
  }

  connectedCallback() {
    super.connectedCallback();
    this.addId();
  }

  createRenderRoot() {
    // Disable shadow dom to use Tailwind CSS
    return this;
  }

  addId() {
    if (this.projects === null) {
      this.projects = [];
    } else {
      this.projects = this.projects.map((item, index) => {
        return { ...item, id: index };
      });
    }
  }

  _getData = () => {
    let item = {
      id: this.projects.length,
      title: "",
      url: "",
      description: "",
      source_url: undefined,
    };

    return item;
  };

  _addProject() {
    this.projects = [...this.projects, this._getData()];
  }

  _removeProject(index) {
    this.projects = this.projects.filter((_, i) => i !== index);
  }

  _onInputChange = (e, index) => {
    const value = e.target.value;
    const name = e.target.dataset.name;

    this.projects[index][name] = value;
  };

  _onTextareaChange = (value, index) => {
    this.projects[index].description = value;
  };

  _getProject(index, project) {
    let data = this._getData(index, project);
    if (project) {
      data = project;
    }

    return html`<div class="mt-10">
      <div class="flex">
        <div
          class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border border-2 border-dashed p-8 rounded-lg bg-gray-50/25 w-2/3"
        >
          <div class="col-span-3">
            <label class="form-label"> Title <span class="asterisk">*</span> </label>
            <div class="mt-2">
              <input
                @input=${(e) => this._onInputChange(e, index)}
                data-name="title"
                type="text"
                name="projects[${index}][title]"
                class="input-primary"
                value="${data.title || ""}"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                required
              />
            </div>
          </div>

          <div class="col-span-3">
            <label class="form-label"> Url <span class="asterisk">*</span> </label>
            <div class="mt-2">
              <input
                @input=${(e) => this._onInputChange(e, index)}
                data-name="url"
                type="url"
                name="projects[${index}][url]"
                class="input-primary"
                value="${data.url || ""}"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                required
              />
            </div>
          </div>

          <div class="col-span-3">
            <label class="form-label"> Source url </label>
            <div class="mt-2">
              <input
                @input=${(e) => this._onInputChange(e, index)}
                data-name="source_url"
                type="url"
                name="projects[${index}][source_url]"
                class="input-primary"
                value="${data.source_url || ""}"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
              />
            </div>
          </div>

          <div class="col-span-full">
            <label for="summary" class="form-label"> Description <span class="asterisk">*</span> </label>
            <div class="mt-2">
              <markdown-editor
                id="projects[${index}][description]"
                name="description"
                content="${data.description || ""}"
                .onChange="${(value) => this._onTextareaChange(value, index)}"
                mini
                required
              ></markdown-editor>
            </div>
          </div>
        </div>

        <div class="ms-3">
          <button
            @click=${() => this._removeProject(index)}
            type="button"
            class="p-3 hover:bg-gray-100 rounded-full"
          >
            <div class="svg-icon size-4 icon-trash bg-gray-600"></div>
          </button>
        </div>
      </div>
    </div>`;
  }

  render() {
    return html`
      <div class="flex items-center space-x-6">
        <div class="text-xl lg:text-2xl font-medium text-gray-900">Projects</div>
        <div>
          <button @click=${this._addProject} type="button" class="group btn-primary-outline btn-mini">
            <div class="flex items-center space-x-1">
              <div class="svg-icon size-2 icon-plus group-hover:bg-white"></div>
              <div>Add</div>
            </div>
          </button>
        </div>
      </div>
      <p class="mt-1 text-sm/6 text-gray-500">
        List interesting projects you have worked on. You can add additional entries by clicking on the
        <span class="italic">Add</span> button next to the title. Entries will be displayed in the order
        provided.
      </p>
      <div id="education-section">
        ${repeat(
          this.projects,
          (d) => d.id,
          (d, index) => this._getProject(index, d),
        )}
      </div>
    `;
  }
}
customElements.define("projects-section", ProjectsSection);
