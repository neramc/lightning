# Actions — Text (canonical spec)

> All ✅ W/M/L/BSD, permission `none`, output as listed.

| id                   | params                                                          | output                             | notes                                                                                          |
| -------------------- | --------------------------------------------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------- |
| `text.text`          | `text` (template)                                               | Text                               | literal/templated value                                                                        |
| `text.combine`       | `separator?`="\n"                                               | Text                               | joins item texts                                                                               |
| `text.split`         | `separator?`="\n" (non-empty)                                   | List<Text>                         |                                                                                                |
| `text.replace`       | `find` · `replace?`="" · `regex?`=false · `caseSensitive?`=true | Text                               | literal mode escapes the pattern and `$` in the replacement; regex mode supports groups (`$1`) |
| `text.change_case`   | `case` enum(lowercase,uppercase,capitalize,sentence)            | Text                               |                                                                                                |
| `text.trim`          | `mode?` enum(both,start,end)=both                               | Text                               |                                                                                                |
| `text.statistics`    | —                                                               | Dictionary{characters,words,lines} |                                                                                                |
| `text.generate_uuid` | —                                                               | Text                               | v4                                                                                             |
| `text.url_encode`    | `mode` enum(encode,decode)                                      | Text                               | percent-encoding                                                                               |
| `text.base64`        | `mode` enum(encode,decode)                                      | Text                               | decode requires UTF-8 payloads                                                                 |
| `text.hash`          | `algorithm` enum(md5,sha1,sha256)                               | Text                               | lowercase hex                                                                                  |

Planned (§8.3 B, not yet implemented — the registry does not expose them):
Match Text · Get Group from Matched Text · Markdown → HTML · HTML → Plain
Text · Translate Text (⚠ requires a configured provider).
