# OCI ID Format

Short base form:

```text
OCI-1-46PK-236
```

Full base form:

```text
OCI-1-46PK-A2-L12-C06
```

Precision offsets append OKLCH deltas with six decimal places:

```text
OCI-1-46PK-236@L+0.002134,C-0.001042,H+0.218400
```

Short and full base IDs map with:

```text
stepNumber = ((anchor - 1) * 120) + ((lightness - 1) * 10) + chroma
```

Offsets store the exact residual from the registered base step. Hue offsets use
the shortest signed hue difference and wrap across 0/360 degrees.
