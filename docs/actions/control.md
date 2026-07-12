# Actions — Control Flow (canonical spec)

> Overrides the §8.3 summary where they differ (CLAUDE.md §18).
> Structural steps (If/Repeat/Repeat with Each/While/Exit/Stop and Output/
> Run Shortcut) are interpreted by the engine; the rest dispatch through the
> registry. All entries: support ✅ W/M/L/BSD, permission `none` unless noted.

## Engine-level steps

| id                     | params                                                                                               | branches            | output                           | notes                                                                  |
| ---------------------- | ---------------------------------------------------------------------------------------------------- | ------------------- | -------------------------------- | ---------------------------------------------------------------------- |
| `control.if`           | `op` enum(equals,notEquals,greaterThan,lessThan,contains,hasValue) · `value` any (omit for hasValue) | `then`, `otherwise` | branch output                    | condition evaluates the step input                                     |
| `control.repeat`       | `count` number ≥ 0                                                                                   | `body`              | List of non-Nothing body outputs | sets `Repeat Index` (1-based); capped by the loop cap (default 10 000) |
| `control.repeat_each`  | `items` any (defaults to input)                                                                      | `body`              | List                             | sets `Repeat Item` + `Repeat Index`                                    |
| `control.while`        | `op`, `value`                                                                                        | `body`              | last body output                 | condition re-evaluated against the current value; loop cap enforced    |
| `control.exit`         | —                                                                                                    | —                   | input                            | unwinds the current shortcut only                                      |
| `control.stop_output`  | `value` any (defaults to input)                                                                      | —                   | value                            |                                                                        |
| `control.run_shortcut` | `shortcut` text (name or id)                                                                         | —                   | child output                     | depth ≤ 16; child Exit does not exit the parent                        |

## Registered actions

| id                           | params                                                   | output         | notes                    |
| ---------------------------- | -------------------------------------------------------- | -------------- | ------------------------ |
| `control.comment`            | `text?`                                                  | passthrough    | never breaks the chain   |
| `control.nothing`            | —                                                        | Nothing        |                          |
| `control.wait`               | `seconds` 0–86400                                        | passthrough    | cancellation-aware sleep |
| `control.get_type`           | —                                                        | Text           | kind display name        |
| `control.count`              | `unit?` enum(items,characters,words,lines)=items         | Number         |                          |
| `control.set_variable`       | `name`                                                   | passthrough    |                          |
| `control.get_variable`       | `name`                                                   | variable value | unknown name ⇒ error     |
| `control.add_to_variable`    | `name`                                                   | List           | creates/wraps as needed  |
| `control.show_result`        | `text?` template                                         | Text           | logs to the run panel    |
| `control.get_item_from_list` | `which` enum(first,last,index,random) · `index?` 1-based | item           | empty list ⇒ Nothing     |
