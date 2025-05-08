# GitJobs

**GitJobs** is a simple open source, developer first job board focused on:

* open source job opportunities filterable by language, technology and foundation
* highlighting the opportunities that contribute back to open source projects
* encouraging sustainability by contributing back to upstream projects

> [!WARNING]
> This project is currently in the early stages of development and is in beta.

| ![Screenshot 1](docs/screenshots/gitjobs1.png?raw=true) | ![Screenshot 2](docs/screenshots/gitjobs2.png?raw=true) |
| ------------------------------------------------------- | ------------------------------------------------------- |
| ![Screenshot 3](docs/screenshots/gitjobs3.png?raw=true) | ![Screenshot 4](docs/screenshots/gitjobs4.png?raw=true) |

## Embed

**GitJobs** allows other websites to embed a view to display the jobs that match certain criteria. You can, for example, list jobs that offer time to work on a specific project, or jobs related to any of the projects in a given foundation. Any criteria supported by the filters, including the text search input, can be used to create an embed view. The embed code is a simple iframe that can be added to any website.

To setup yours, the first step is to adjust the filters as if you were searching for jobs. Once you have the desired filters, you can click on the `Get embed code` button, at the bottom of the filters. This will open a modal with the ready to use embed code.

You can see it in action in this [example](https://codepen.io/cynthiasg/pen/gbOJLOb).

### Embedding a single job

It is also possible to **embed a single job**. To do this, you can click on the `Get embed code` button in the job details page (in the *share* section). The embed code is available in *Markdown*, *AsciiDoc* and *HTML* formats. The job card generated is an SVG image, and the code will include a link to the job at <https://gitjobs.dev>.

![Single job embed](docs/screenshots/embed_job.png?raw=true)

## Contributing

Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.

## Code of Conduct

This project follows the [CNCF Code of Conduct](https://github.com/cncf/foundation/blob/master/code-of-conduct.md).

## License

**GitJobs** is an open source project licensed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).
