# Limitations

OCI v1-beta is a reference implementation, not a frozen production standard.

## Not Implemented

- Pantone official libraries.
- RAL official libraries.
- Numeric CMYK conversion.
- ICC profile conversion.
- Gamut mapping.
- GUI.
- Web app.
- Cloud service.
- Plugins.
- Physical print proofing automation.

## CMYK

CMYK returns:

```text
profile_required
```

No numeric CMYK value is produced. CMYK requires ICC profile support and print
workflow decisions.

## Pantone And RAL

Pantone and RAL are not included. There is no official lookup table and no
approximation output in v1-beta.

## Gamut Mapping

If a target color system cannot represent a canonical OKLCH color and no gamut
mapping is implemented, the target status is `unsupported`.

The implementation does not silently clamp and call that supported.

## Registry Stability

Families and steps are frozen runtime data. The generator can rebuild
`registry/v1/*.json`, but the OCI kernel does not regenerate steps at runtime.

## Family Names

Family names are stable labels, not subjective color judgments. The family
classifier uses deterministic numeric rules, and earth/muted/neutral classes are
semantic zones rather than ordinary equal-width hue sectors.

## HEX

HEX is lossy 8-bit sRGB. It can be useful and stable as a representation, but
it is not the canonical identity model.

## Print

Physical print proofing is still required for production.
