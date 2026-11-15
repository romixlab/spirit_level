## Why

A reader who meets an abbreviation they do not know stops, searches the page for its meaning and often finds none. Readers arrive on any page from a search or a link, so each page spells out its abbreviations once, at the first use.

The lint reads a word of two to six capitals and digits, with at least two capitals and at most one digit, as an abbreviation: `PWM`, `I2C`, `MCUs`. A word with more digits is a part number (`STM32`). It counts as spelled out when the words stand next to it in parentheses, either way round, or when "stands for", "short for" or "means" follows it. Abbreviations every reader knows are in the `known` list and never judged. A file name such as `AGENTS.md`, inline code and words written in capitals for stress (`MUST`, `NOT`) are not abbreviations.

The finding is a candidate: some words in capitals are names, not abbreviations.

```markdown
The MCU drives the motor.                                   <!-- flagged -->
The microcontroller (MCU) drives the motor.                 <!-- spelled out -->
```

## How to fix

Write the words out at the first use, the abbreviation in parentheses after them. An abbreviation that every reader of the project knows goes into the `known` list of the project's `spirit_level.toml`.
