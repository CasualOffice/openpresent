# LINE: Lines and arrowheads

Generated. Tier 1E; render 2B; edit 4; level Edit. 95 rows.


## `a:ST_LineEndType`
Restriction: `base=token`
- `none`
- `triangle`
- `stealth`
- `diamond`
- `oval`
- `arrow`

## `a:ST_LineEndWidth`
Restriction: `base=token`
- `sm`
- `med`
- `lg`

## `a:ST_LineEndLength`
Restriction: `base=token`
- `sm`
- `med`
- `lg`

## `a:CT_LineEndProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `ST_LineEndType` | optional | none |  |
| attribute | `@w` | `ST_LineEndWidth` | optional |  |  |
| attribute | `@len` | `ST_LineEndLength` | optional |  |  |

## `a:EG_LineFillProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `noFill` | `CT_NoFillProperties` | choice |  |  |
| element | `solidFill` | `CT_SolidColorFillProperties` | choice |  |  |
| element | `gradFill` | `CT_GradientFillProperties` | choice |  |  |
| element | `pattFill` | `CT_PatternFillProperties` | choice |  |  |

## `a:CT_LineJoinBevel`

## `a:CT_LineJoinRound`

## `a:CT_LineJoinMiterProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@lim` | `ST_PositivePercentage` | optional |  |  |

## `a:EG_LineJoinProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `round` | `CT_LineJoinRound` | choice |  |  |
| element | `bevel` | `CT_LineJoinBevel` | choice |  |  |
| element | `miter` | `CT_LineJoinMiterProperties` | choice |  |  |

## `a:ST_PresetLineDashVal`
Restriction: `base=token`
- `solid`
- `dot`
- `dash`
- `lgDash`
- `dashDot`
- `lgDashDot`
- `lgDashDotDot`
- `sysDash`
- `sysDot`
- `sysDashDot`
- `sysDashDotDot`

## `a:CT_PresetLineDashProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_PresetLineDashVal` | optional |  |  |

## `a:CT_DashStop`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@d` | `ST_PositivePercentage` | required |  |  |
| attribute | `@sp` | `ST_PositivePercentage` | required |  |  |

## `a:CT_DashStopList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ds` | `CT_DashStop` | 0..unbounded |  |  |

## `a:EG_LineDashProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `prstDash` | `CT_PresetLineDashProperties` | choice |  |  |
| element | `custDash` | `CT_DashStopList` | choice |  |  |

## `a:ST_LineCap`
Restriction: `base=token`
- `rnd`
- `sq`
- `flat`

## `a:ST_LineWidth`
Restriction: `base=ST_Coordinate32Unqualified minInclusive=0 maxInclusive=20116800`

## `a:ST_PenAlignment`
Restriction: `base=token`
- `ctr`
- `in`

## `a:ST_CompoundLine`
Restriction: `base=token`
- `sng`
- `dbl`
- `thickThin`
- `thinThick`
- `tri`

## `a:CT_LineProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_LineFillProperties` | `EG_LineFillProperties` | 0..1 |  |  |
| group | `EG_LineDashProperties` | `EG_LineDashProperties` | 0..1 |  |  |
| group | `EG_LineJoinProperties` | `EG_LineJoinProperties` | 0..1 |  |  |
| element | `headEnd` | `CT_LineEndProperties` | 0..1 |  |  |
| element | `tailEnd` | `CT_LineEndProperties` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@w` | `ST_LineWidth` | optional |  |  |
| attribute | `@cap` | `ST_LineCap` | optional |  |  |
| attribute | `@cmpd` | `ST_CompoundLine` | optional |  |  |
| attribute | `@algn` | `ST_PenAlignment` | optional |  |  |

## `x:drawing/2018/sketchyshapes:CT_Empty`

## `x:drawing/2018/sketchyshapes:ST_LineSketchSeed`
Restriction: `base=unsignedInt`

## `x:drawing/2018/sketchyshapes:EG_LineSketchType`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `lineSketchNone` | `CT_Empty` | choice |  |  |
| element | `lineSketchCurved` | `CT_Empty` | choice |  |  |
| element | `lineSketchFreehand` | `CT_Empty` | choice |  |  |
| element | `lineSketchScribble` | `CT_Empty` | choice |  |  |

## `x:drawing/2018/sketchyshapes:CT_LineSketchTypeProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_LineSketchType` | `EG_LineSketchType` | 0..1 |  |  |

## `x:drawing/2018/sketchyshapes:CT_LineSketchStyleProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_Geometry` | `EG_Geometry` | 0..1 |  |  |
| element | `type` | `CT_LineSketchTypeProperties` | 0..1 |  |  |
| element | `seed` | `ST_LineSketchSeed` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@sd` | `ST_LineSketchSeed` | optional |  |  |

## `x:drawing/2018/sketchyshapes:<lineSketchStyleProps>` : `CT_LineSketchStyleProperties`
