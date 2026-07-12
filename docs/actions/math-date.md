# Actions — Math & Date (canonical spec)

> All ✅ W/M/L/BSD, permission `none`.

## Math

| id                     | params                                                                         | output | notes                                                      |
| ---------------------- | ------------------------------------------------------------------------------ | ------ | ---------------------------------------------------------- |
| `math.calculate`       | `operation` enum(add,subtract,multiply,divide,modulo,power) · `operand` number | Number | input is the left operand; ÷0 and non-finite results fail  |
| `math.random`          | `min` · `max` · `integer?`=true                                                | Number | uniform, inclusive                                         |
| `math.round`           | `mode?` enum(nearest,up,down)=nearest · `decimals?`=0 (0–12)                   | Number |                                                            |
| `math.list_statistics` | `operation` enum(minimum,maximum,average,sum,median,count)                     | Number | items coerce to numbers; empty list fails except sum/count |

## Date & Time

| id                  | params                                                                                         | output | notes                                              |
| ------------------- | ---------------------------------------------------------------------------------------------- | ------ | -------------------------------------------------- |
| `date.current`      | —                                                                                              | Date   | UTC instant; UI renders per locale (ICU)           |
| `date.adjust`       | `amount` · `unit` enum(seconds,minutes,hours,days,weeks) · `direction?` enum(add,subtract)=add | Date   | month/year arithmetic ships with the calendar work |
| `date.format`       | `format` (chrono strftime)                                                                     | Text   | invalid patterns rejected up front                 |
| `date.time_between` | `other` date · `unit?` enum(seconds,minutes,hours,days)=seconds                                | Number | absolute difference                                |

## Scripting

| id                         | params                                                            | output  | notes                                                                                        |
| -------------------------- | ----------------------------------------------------------------- | ------- | -------------------------------------------------------------------------------------------- |
| `scripting.run_javascript` | `script` (reviewed in full per §14) · `timeoutMs?`=5000 (1–60000) | dynamic | QuickJS sandbox: no fs/net/process; `input` global + `console.log` → run log; 32 MB heap cap |
