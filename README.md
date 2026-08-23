<div align="center">

Dialogue Editor
===============

[![pipeline status](https://gitlab.com/245project/tools/dialogue-editor/badges/master/pipeline.svg)](https://gitlab.com/245project/tools/dialogue-editor/-/commits/master)

[![Gitlab](https://img.shields.io/badge/gitlab-%23181717.svg?style=for-the-badge&logo=gitlab&logoColor=white)](https://gitlab.com/245project/tools/dialogue-editor)
[![Github](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)](https://github.com/dothanhtrung/dialogue-editor)

![](./assets/screenshot.png)

</div>

Dialogue editor for [bevy-dialogue](https://gitlab.com/245project/bevy-plugin/bevy-dialogue).

> Although the file schema is stable, the editor is still in progress.


TODO:

* High priority
    * [ ] Reorder class and state
    * [ ] Group class/sequence

* Low priority
    * [ ] Change font
    * [ ] Disable dialogue without deletion
    * [ ] Dialogue condition
    * [ ] Metadata (e.g. schema version)
    * [ ] Resizable UI

* Very low priority
    * [ ] Workspace
        * [ ] Open multiple files
    * [ ] Graph visualization


Data structure
--------------

```ron
(
    dialogues: {
        <class_id>: {
            <state_id>: [
                (
                    contents: {
                        "<language code>": "Dialogue content. {{variable}} is supported",
                    },
                    affects: {
                        <target_class_id>: <target_state_id>,
                    },
                    events: [event_id]
                ),
            ],
        },
    },
    sequences: {
        <sequence_id>>: [
            (
                class: <class_id>,
                state: <state_id>,
                dialogue: Option<usize>,
            ),
        ],
    },
)
```

| Field        | Type   | Description                                                                                                                          |
|:------------ |:------ |:------------------------------------------------------------------------------------------------------------------------------------ |
|class_id      |u64     |The character class id. For example: Villager, Hero, etc. should have unique id.                                                      |
|state_id      |u64     |The character state id. For example: Idle, Arguing, Cheering, etc. should have unique id.                                             |
|language_code |String  |3 character language code by ISO 639-3. For example: `eng`, `spa`, etc.                                                               |
|affects       |HashMap |If a character with `target_class_id` talks to the one with this dialogue, that character state will be changed to `target_state_id`. |
|events        |[u64]   |Array of event id. They will be triggered by plugin if the dialogue is used.                                                          |
|sequence_id   |u64     |ID if the sequence. Sequence is a list of dialogues to be displayed in order. If `dialogue` is not specified, all dialogues in same state will be used. |


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

### Output

This application can output file in 2 formats: `.ron` a text file, and `.bin` a binary file, depending on your file name when save.

License
-------

Please see [LICENSE.md](./LICENSE.md).

<div align="center">

![git_dialogue-editor](https://count.getloli.com/@git_dialogue-editor?name=git_dialogue-editor&theme=random&padding=10&offset=0&align=top&scale=1&pixelated=1&darkmode=auto)

</div>
