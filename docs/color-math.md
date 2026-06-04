# Color Math

The core uses typed structs and `f64` internally. It does not pass raw
`[f64; 3]` values through the public core.

## Typed Color Structs

RGB-like spaces distinguish encoded and linear values:

- `EncodedSrgb`
- `LinearSrgb`
- `EncodedDisplayP3`
- `LinearDisplayP3`
- `EncodedAdobeRgb1998`
- `LinearAdobeRgb1998`
- `EncodedRec709`
- `LinearRec709`

Canonical and connection spaces:

- `XyzD65`
- `Oklab`
- `Oklch`

Every RGB-like type has:

```rust
new(r: f64, g: f64, b: f64) -> Self
is_in_gamut(self) -> bool
```

Gamut check accepts finite channels in:

```text
[-1e-12, 1 + 1e-12]
```

## HEX And RGB Parsing

HEX parsing accepts:

```text
#RRGGBB
RRGGBB
#RGB
RGB
```

Short HEX expands each digit:

```text
#0f8 -> #00ff88
```

8-bit sRGB converts to encoded float sRGB:

```text
channelFloat = channelU8/255
```

## HSL To sRGB

For HSL input, hue wraps by `rem_euclid(360)`, saturation and lightness must be
in `0.0..=1.0`.

If `S = 0`, output is grayscale:

```text
R = G = B = L
```

Otherwise:

```text
Hn = (H mod 360)/360

if L < 0.5:
  q = L * (1 + S)
else:
  q = L + S - L * S

p = 2L - q
R = hueToRgb(p, q, Hn + 1/3)
G = hueToRgb(p, q, Hn)
B = hueToRgb(p, q, Hn - 1/3)
```

`hueToRgb` is the standard piecewise interpolation over the unit hue circle.

## Transfer Functions

### sRGB/Display P3

Display P3 uses the same transfer function as sRGB in this implementation.

Decode encoded channel `u` to linear:

```text
if u <= 0.04045:
  linear = u/12.92
else:
  linear = ((u + 0.055)/1.055)^2.4
```

Encode linear channel `x`:

```text
if x <= 0.0031308:
  encoded = 12.92 * x
else:
  encoded = 1.055 * x^(1/2.4) - 0.055
```

### Adobe RGB 1998

Gamma:

```text
gamma = 563/256
```

Decode:

```text
linear = signedPow(encoded, gamma)
```

Encode:

```text
encoded = signedPow(linear, 1/gamma)
```

`signedPow` preserves sign for negative intermediate values:

```text
signedPow(v, p) = if v < 0 then -((-v)^p) else v^p
```

### Rec.709

Decode encoded channel `u`:

```text
if u < 0.081:
  linear = u/4.5
else:
  linear = ((u + 0.099)/1.099)^(1/0.45)
```

Encode linear channel `x`:

```text
if x < 0.018:
  encoded = 4.5 * x
else:
  encoded = 1.099 * x^0.45 - 0.099
```

## RGB To XYZ D65 Matrices

The matrix operation is:

```text
X = m11*R + m12*G + m13*B
Y = m21*R + m22*G + m23*B
Z = m31*R + m32*G + m33*B
```

### sRGB/Rec.709 Linear RGB To XYZ

```text
[ 0.4124564  0.3575761  0.1804375 ]
[ 0.2126729  0.7151522  0.0721750 ]
[ 0.0193339  0.1191920  0.9503041 ]
```

### XYZ To sRGB/Rec.709 Linear RGB

```text
[  3.2404542  -1.5371385  -0.4985314 ]
[ -0.9692660   1.8760108   0.0415560 ]
[  0.0556434  -0.2040259   1.0572252 ]
```

### Display P3 Linear RGB To XYZ

```text
[ 0.4865709486482162  0.26566769316909306  0.1982172852343625 ]
[ 0.2289745640697488  0.6917385218365064   0.0792869140937450 ]
[ 0.0000000000000000  0.04511338185890264  1.0439443689009760 ]
```

### XYZ To Display P3 Linear RGB

```text
[  2.4934969119414250  -0.9313836179191239  -0.40271078445071684 ]
[ -0.8294889695615747   1.7626640603183463   0.023624685841943577 ]
[  0.03584583024378447 -0.07617238926804182  0.9568845240076872 ]
```

### Adobe RGB 1998 Linear RGB To XYZ

```text
[ 0.5767309  0.1855540  0.1881852 ]
[ 0.2973769  0.6273491  0.0752741 ]
[ 0.0270343  0.0706872  0.9911085 ]
```

### XYZ To Adobe RGB 1998 Linear RGB

```text
[  2.0413690  -0.5649464  -0.3446944 ]
[ -0.9692660   1.8760108   0.0415560 ]
[  0.0134474  -0.1183897   1.0154096 ]
```

## XYZ D65 To OKLab

Given XYZ:

```text
l = 0.8189330101X + 0.3618667424Y - 0.1288597137Z
m = 0.0329845436X + 0.9293118715Y + 0.0361456387Z
s = 0.0482003018X + 0.2643662691Y + 0.6338517070Z

l_ = cbrt(l)
m_ = cbrt(m)
s_ = cbrt(s)

L = 0.2104542553l_ + 0.7936177850m_ - 0.0040720468s_
a = 1.9779984951l_ - 2.4285922050m_ + 0.4505937099s_
b = 0.0259040371l_ + 0.7827717662m_ - 0.8086757660s_
```

## OKLab To XYZ D65

```text
l_ = L + 0.3963377774a + 0.2158037573b
m_ = L - 0.1055613458a - 0.0638541728b
s_ = L - 0.0894841775a - 1.2914855480b

l = l_^3
m = m_^3
s = s_^3

X =  1.2270138511l - 0.5577999807m + 0.2812561490s
Y = -0.0405801784l + 1.1122568696m - 0.0716766787s
Z = -0.0763812845l - 0.4214819784m + 1.5861632204s
```

## OKLab And OKLCH

OKLab to OKLCH:

```text
L = L
C = sqrt(a^2 + b^2)
H = atan2(b, a) in degrees, wrapped to [0, 360)
```

OKLCH to OKLab:

```text
h = (H mod 360) * pi/180
L = L
a = C * cos(h)
b = C * sin(h)
```

## Quantization

HEX and 8-bit RGB clamp encoded sRGB to `0.0..=1.0`, multiply by 255, and
round:

```text
u8 = round(clamp(channel, 0, 1) * 255)
```

HEX is uppercase:

```text
#E85A9A
```
