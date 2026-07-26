<div align="center">

Dialogue Editor
===============

[![pipeline status](https://gitlab.com/245project/tools/dialogue-editor/badges/master/pipeline.svg)](https://gitlab.com/245project/tools/dialogue-editor/-/commits/master)

[![Gitlab](https://img.shields.io/badge/gitlab-%23181717.svg?style=for-the-badge&logo=gitlab&logoColor=white)](https://gitlab.com/245project/tools/dialogue-editor)
[![Github](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)](https://github.com/dothanhtrung/dialogue-editor)
</div>

Dialogue editor for [bevy-dialogue](https://gitlab.com/245project/bevy-plugin/bevy-dialogue).

> Although the file schema is stable, the editor is still in progress.

TODO:

* High priority
    * [ ] Better UI
    * [ ] Reorder class and state

* Low priority
    * [ ] Change font
    * [ ] Disable dialogue without deletion
    * [ ] Dialogue condition
    * [ ] Metadata (e.g. schema version)
    * [ ] Resizable UI
    * [ ] Autosave

* Very low priority
    * [ ] Workspace
        * [ ] Open multiple files
    * [ ] Graph visualization

<div align="center">

![](./assets/screenshot.png)
</div>

How to
------

### Build

```shell
cargo build --release
```

Output: `target/release/dialogue-editor`.

### Run

```shell
./dialogue-editor -c dialogue-editor.ron
```
* `dialogue-editor.ron`: Application config file.

License
-------

Please see [LICENSE.md](./LICENSE.md).

<div align="center">

![git_dialogue-editor](https://count.getloli.com/@git_dialogue-editor?name=git_dialogue-editor&theme=random&padding=10&offset=0&align=top&scale=1&pixelated=1&darkmode=auto)

</div>
