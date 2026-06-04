# Registry Data

The registry is frozen runtime data. The runtime library reads included JSON
files and does not regenerate registered steps dynamically.

Registry invariants:
- 64 families.
- 360 steps per family.
- 23,040 registered base steps.
- Three hue anchors per family.
- Twelve lightness levels and ten chroma levels.

Family names are stable labels, not subjective color judgments. Registered
families and steps are frozen data at runtime.

Family classification is split by class:
- `chromatic` families are ordinary hue-based families.
- `earth_muted` families are selected from semantic lightness, chroma, and hue
  zones such as brown, copper, tan, beige, olive, olive green, maroon red, and
  maroon.
- `neutral` families are selected by very low chroma plus lightness conditions
  for black, white, gray, neutral gray, slate gray, and slate.

Earth/muted and neutral families do not own ordinary equal-width hue sectors.
High-chroma input colors must remain in chromatic families and must not encode
to grayscale or neutral families.
