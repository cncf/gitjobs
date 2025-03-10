import { LitElement, html, repeat } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";
import { isObjectEmpty } from "/static/js/common/common.js";

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
    if (this.children.length === 0) {
      // Disable shadow dom to use Tailwind CSS
      return this;
    } else {
      // Remove previous content when re-rendering full component
      this.innerHTML = "";
      // Disable shadow dom to use Tailwind CSS
      return this;
    }
  }

  addId() {
    if (this.projects === null) {
      this.projects = [this._getData()];
    } else {
      this.projects = this.projects.map((item, index) => {
        return { ...item, id: index };
      });
    }
  }

  _getData = () => {
    let item = {
      id: this.projects ? this.projects.length : 0,
      title: "",
      url: "",
      description: "",
      source_url: "",
    };

    return item;
  };

  _addProject(index) {
    const currentProjects = [...this.projects];
    currentProjects.splice(index, 0, this._getData());

    this.projects = currentProjects;
  }

  _removeProject(index) {
    const tmpProjects = this.projects.filter((_, i) => i !== index);
    // If there are no more projects, add a new one
    this.projects = tmpProjects.length === 0 ? [this._getData()] : tmpProjects;
  }

  _onDataChange = (data, index) => {
    this.projects[index] = data;
  };

  _getProject(index, project) {
    const hasSingleProject = this.projects.length === 1;

    return html`<div class="mt-10">
      <div class="flex w-3/4 lg:w-2/3">
        <div class="flex flex-col space-y-3 me-3">
          <div>
            <button
              @click=${() => this._addProject(index)}
              type="button"
              class="p-2 border hover:bg-gray-100 rounded-full"
              title="Add above"
            >
              <div class="svg-icon size-4 icon-plus_top bg-gray-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._addProject(index + 1)}
              type="button"
              class="p-2 border hover:bg-gray-100 rounded-full"
              title="Add below"
            >
              <div class="svg-icon size-4 icon-plus_bottom bg-gray-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._removeProject(index)}
              type="button"
              class="p-2 border hover:bg-gray-100 rounded-full"
              title="${hasSingleProject ? "Clean" : "Delete"}"
            >
              <div class="svg-icon size-4 icon-${hasSingleProject ? "eraser" : "trash"} bg-gray-600"></div>
            </button>
          </div>
        </div>
        <project-entry
          .data=${project}
          .index=${index}
          .onDataChange=${this._onDataChange}
          class="w-full"
        ></project-entry>
      </div>
    </div>`;
  }

  render() {
    return html`
      <div class="text-sm/6 text-gray-500">
        List interesting projects you have worked on. You can add additional entries by clicking on the
        <span class="font-semibold">+</span> buttons on the left of the card (
        <div class="inline-block svg-icon size-4 icon-plus_top bg-gray-600 relative -bottom-[2px]"></div>
        to add the new entry above,
        <div class="inline-block svg-icon size-4 icon-plus_bottom bg-gray-600 relative -bottom-[2px]"></div>
        to add it below). Entries will be displayed in the order provided.
      </div>
      <div id="projects-section">
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

class Project extends LitElement {
  static properties = {
    data: { type: Object },
    index: { type: Number },
    isObjectEmpty: { type: Boolean },
    onDataChange: { type: Function },
  };

  constructor() {
    super();
    this.data = {
      id: 0,
      title: "",
      url: "",
      description: "",
      source_url: "",
    };
    this.index = 0;
    this.isObjectEmpty = true;
    this.onDataChange = () => {};
  }

  createRenderRoot() {
    // Disable shadow dom to use Tailwind CSS
    return this;
  }

  connectedCallback() {
    super.connectedCallback();
    this.isObjectEmpty = isObjectEmpty(this.data);
  }

  _onInputChange = (e) => {
    const value = e.target.value;
    const name = e.target.dataset.name;

    this.data[name] = value;
    this.isObjectEmpty = isObjectEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  _onTextareaChange = (value) => {
    this.data.description = value;
    this.isObjectEmpty = isObjectEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  render() {
    return html`<div
      class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border border-2 border-dashed p-8 rounded-lg bg-gray-50/25 w-full"
    >
      <div class="col-span-3">
        <label class="form-label"> Title <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${(e) => this._onInputChange(e)}
            data-name="title"
            type="text"
            name="projects[${this.index}][title]"
            class="input-primary"
            value="${this.data.title}"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            ?required=${!this.isObjectEmpty}
          />
        </div>
      </div>

      <div class="col-span-3"></div>

      <div class="col-span-3">
        <label class="form-label"> Url <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${(e) => this._onInputChange(e)}
            data-name="url"
            type="url"
            name="projects[${this.index}][url]"
            class="input-primary"
            value="${this.data.url}"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            ?required=${!this.isObjectEmpty}
          />
        </div>
      </div>

      <div class="col-span-3">
        <label class="form-label"> Source url </label>
        <div class="mt-2">
          <input
            @input=${(e) => this._onInputChange(e)}
            data-name="source_url"
            type="url"
            name="projects[${this.index}][source_url]"
            class="input-primary"
            value="${this.data.source_url}"
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
            id="projects[${this.index}][description]"
            name="description"
            content="${this.data.description}"
            .onChange="${(value) => this._onTextareaChange(value)}"
            mini
            ?required=${!this.isObjectEmpty}
          ></markdown-editor>
        </div>
      </div>
    </div>`;
  }
}
customElements.define("project-entry", Project);
