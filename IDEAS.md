# Ideas

Running list of features, experiments, and stretch goals for the delay plugin.

## Features

- **Delay mode toggle: Tape vs. Clean**
  - *Tape*: read pointer moves continuously when delay time changes → pitch-shift / Doppler artifacts (the classic tape-delay sound).
  - *Clean*: crossfade between two read pointers (old position fading out, new fading in over ~5–20 ms) → no pitch shift, just a brief dissolve.
  - Exposed as a toggle/button parameter (likely a `BoolParam` or `EnumParam` if more modes get added later).
