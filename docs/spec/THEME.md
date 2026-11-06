# THEME: Themes, format schemes, colour maps

Generated. Tier 1C; render 2A; edit 7; level Edit. 159 rows.


## `a:ST_StyleMatrixColumnIndex`
Restriction: `base=unsignedInt`

## `a:ST_FontCollectionIndex`
Restriction: `base=token`
- `major`
- `minor`
- `none`

## `a:ST_ColorSchemeIndex`
Restriction: `base=token`
- `dk1`
- `lt1`
- `dk2`
- `lt2`
- `accent1`
- `accent2`
- `accent3`
- `accent4`
- `accent5`
- `accent6`
- `hlink`
- `folHlink`

## `a:CT_ColorScheme`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `dk1` | `CT_Color` |  |  |  |
| element | `lt1` | `CT_Color` |  |  |  |
| element | `dk2` | `CT_Color` |  |  |  |
| element | `lt2` | `CT_Color` |  |  |  |
| element | `accent1` | `CT_Color` |  |  |  |
| element | `accent2` | `CT_Color` |  |  |  |
| element | `accent3` | `CT_Color` |  |  |  |
| element | `accent4` | `CT_Color` |  |  |  |
| element | `accent5` | `CT_Color` |  |  |  |
| element | `accent6` | `CT_Color` |  |  |  |
| element | `hlink` | `CT_Color` |  |  |  |
| element | `folHlink` | `CT_Color` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | required |  |  |

## `a:CT_CustomColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` |  |  |  |
| attribute | `@name` | `string` | optional |  |  |

## `a:CT_SupplementalFont`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@script` | `string` | required |  |  |
| attribute | `@typeface` | `ST_TextTypeface` | required |  |  |

## `a:CT_CustomColorList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `custClr` | `CT_CustomColor` | 0..unbounded |  |  |

## `a:CT_FontCollection`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `latin` | `CT_TextFont` |  |  |  |
| element | `ea` | `CT_TextFont` |  |  |  |
| element | `cs` | `CT_TextFont` |  |  |  |
| element | `font` | `CT_SupplementalFont` | 0..unbounded |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_EffectStyleItem`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_EffectProperties` | `EG_EffectProperties` |  |  |  |
| element | `scene3d` | `CT_Scene3D` | 0..1 |  |  |
| element | `sp3d` | `CT_Shape3D` | 0..1 |  |  |

## `a:CT_FontScheme`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `majorFont` | `CT_FontCollection` |  |  |  |
| element | `minorFont` | `CT_FontCollection` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | required |  |  |

## `a:CT_FillStyleList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_FillProperties` | `EG_FillProperties` | 3..unbounded |  |  |

## `a:CT_LineStyleList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ln` | `CT_LineProperties` | 3..unbounded |  |  |

## `a:CT_EffectStyleList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `effectStyle` | `CT_EffectStyleItem` | 3..unbounded |  |  |

## `a:CT_BackgroundFillStyleList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_FillProperties` | `EG_FillProperties` | 3..unbounded |  |  |

## `a:CT_StyleMatrix`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `fillStyleLst` | `CT_FillStyleList` |  |  |  |
| element | `lnStyleLst` | `CT_LineStyleList` |  |  |  |
| element | `effectStyleLst` | `CT_EffectStyleList` |  |  |  |
| element | `bgFillStyleLst` | `CT_BackgroundFillStyleList` |  |  |  |
| attribute | `@name` | `string` | optional |  |  |

## `a:CT_BaseStyles`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clrScheme` | `CT_ColorScheme` |  |  |  |
| element | `fontScheme` | `CT_FontScheme` |  |  |  |
| element | `fmtScheme` | `CT_StyleMatrix` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_EffectReference`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@ref` | `token` | required |  |  |

## `a:CT_StyleMatrixReference`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 0..1 |  |  |
| attribute | `@idx` | `ST_StyleMatrixColumnIndex` | required |  |  |

## `a:CT_FontReference`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 0..1 |  |  |
| attribute | `@idx` | `ST_FontCollectionIndex` | required |  |  |

## `a:CT_ShapeStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `lnRef` | `CT_StyleMatrixReference` |  |  |  |
| element | `fillRef` | `CT_StyleMatrixReference` |  |  |  |
| element | `effectRef` | `CT_StyleMatrixReference` |  |  |  |
| element | `fontRef` | `CT_FontReference` |  |  |  |

## `a:CT_DefaultShapeDefinition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `bodyPr` | `CT_TextBodyProperties` |  |  |  |
| element | `lstStyle` | `CT_TextListStyle` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_ObjectStyleDefaults`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spDef` | `CT_DefaultShapeDefinition` | 0..1 |  |  |
| element | `lnDef` | `CT_DefaultShapeDefinition` | 0..1 |  |  |
| element | `txDef` | `CT_DefaultShapeDefinition` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_EmptyElement`

## `a:CT_ColorMapping`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@bg1` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@tx1` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@bg2` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@tx2` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@accent1` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@accent2` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@accent3` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@accent4` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@accent5` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@accent6` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@hlink` | `ST_ColorSchemeIndex` | required |  |  |
| attribute | `@folHlink` | `ST_ColorSchemeIndex` | required |  |  |

## `a:CT_ColorMappingOverride`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `masterClrMapping` | `CT_EmptyElement` | choice |  |  |
| element | `overrideClrMapping` | `CT_ColorMapping` | choice |  |  |

## `a:CT_ColorSchemeAndMapping`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clrScheme` | `CT_ColorScheme` |  |  |  |
| element | `clrMap` | `CT_ColorMapping` | 0..1 |  |  |

## `a:CT_ColorSchemeList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extraClrScheme` | `CT_ColorSchemeAndMapping` | 0..unbounded |  |  |

## `a:CT_OfficeStyleSheet`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `themeElements` | `CT_BaseStyles` |  |  |  |
| element | `objectDefaults` | `CT_ObjectStyleDefaults` | 0..1 |  |  |
| element | `extraClrSchemeLst` | `CT_ColorSchemeList` | 0..1 |  |  |
| element | `custClrLst` | `CT_CustomColorList` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | optional |  |  |

## `a:CT_BaseStylesOverride`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clrScheme` | `CT_ColorScheme` | 0..1 |  |  |
| element | `fontScheme` | `CT_FontScheme` | 0..1 |  |  |
| element | `fmtScheme` | `CT_StyleMatrix` | 0..1 |  |  |

## `a:CT_ClipboardStyleSheet`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `themeElements` | `CT_BaseStyles` |  |  |  |
| element | `clrMap` | `CT_ColorMapping` |  |  |  |

## `a:<theme>` : `CT_OfficeStyleSheet`

## `a:<themeOverride>` : `CT_BaseStylesOverride`

## `a:<themeManager>` : `CT_EmptyElement`

## `a:EG_ThemeableFillStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `fill` | `CT_FillProperties` | choice |  |  |
| element | `fillRef` | `CT_StyleMatrixReference` | choice |  |  |

## `a:EG_ThemeableEffectStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `effect` | `CT_EffectProperties` | choice |  |  |
| element | `effectRef` | `CT_StyleMatrixReference` | choice |  |  |

## `a:EG_ThemeableFontStyles`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `font` | `CT_FontCollection` | choice |  |  |
| element | `fontRef` | `CT_FontReference` | choice |  |  |

## `a:ST_OnOffStyleType`
Restriction: `base=token`
- `on`
- `off`
- `def`

## `x:thememl/2012/main:CT_ThemeFamily`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | required |  |  |
| attribute | `@id` | `ST_Guid` | required |  |  |
| attribute | `@vid` | `ST_Guid` | required |  |  |

## `x:thememl/2012/main:<themeFamily>` : `CT_ThemeFamily`

## `x:thememl/2012/main:@id` : `ST_Guid`
