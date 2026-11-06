# SIMPLE: Shared value types

Generated. Tier 1A; render -; edit -; level Edit. 93 rows.


## `st:ST_Lang`
Restriction: `base=string`

## `st:ST_HexColorRGB`
Restriction: `base=hexBinary length=3`

## `st:ST_Panose`
Restriction: `base=hexBinary length=10`

## `st:ST_CalendarType`
Restriction: `base=string`
- `gregorian`
- `gregorianUs`
- `gregorianMeFrench`
- `gregorianArabic`
- `hijri`
- `hebrew`
- `taiwan`
- `japan`
- `thai`
- `korea`
- `saka`
- `gregorianXlitEnglish`
- `gregorianXlitFrench`
- `none`

## `st:ST_AlgClass`
Restriction: `base=string`
- `hash`
- `custom`

## `st:ST_CryptProv`
Restriction: `base=string`
- `rsaAES`
- `rsaFull`
- `custom`

## `st:ST_AlgType`
Restriction: `base=string`
- `typeAny`
- `custom`

## `st:ST_ColorType`
Restriction: `base=string`

## `st:ST_Guid`
Restriction: `base=token pattern=\{[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}\}`

## `st:ST_OnOff`
Restriction: `union=boolean ST_OnOff1`

## `st:ST_OnOff1`
Restriction: `base=string`
- `on`
- `off`

## `st:ST_String`
Restriction: `base=string`

## `st:ST_XmlName`
Restriction: `base=NCName minLength=1 maxLength=255`

## `st:ST_TrueFalse`
Restriction: `base=string`
- `t`
- `f`
- `true`
- `false`

## `st:ST_TrueFalseBlank`
Restriction: `base=string`
- `t`
- `f`
- `true`
- `false`
- ``
- `True`
- `False`

## `st:ST_UnsignedDecimalNumber`
Restriction: `base=unsignedLong`

## `st:ST_TwipsMeasure`
Restriction: `union=ST_UnsignedDecimalNumber ST_PositiveUniversalMeasure`

## `st:ST_VerticalAlignRun`
Restriction: `base=string`
- `baseline`
- `superscript`
- `subscript`

## `st:ST_Xstring`
Restriction: `base=string`

## `st:ST_XAlign`
Restriction: `base=string`
- `left`
- `center`
- `right`
- `inside`
- `outside`

## `st:ST_YAlign`
Restriction: `base=string`
- `inline`
- `top`
- `center`
- `bottom`
- `inside`
- `outside`

## `st:ST_ConformanceClass`
Restriction: `base=string`
- `strict`
- `transitional`

## `st:ST_UniversalMeasure`
Restriction: `base=string pattern=-?[0-9]+(\.[0-9]+)?(mm\|cm\|in\|pt\|pc\|pi)`

## `st:ST_PositiveUniversalMeasure`
Restriction: `base=ST_UniversalMeasure pattern=[0-9]+(\.[0-9]+)?(mm\|cm\|in\|pt\|pc\|pi)`

## `st:ST_Percentage`
Restriction: `base=string pattern=-?[0-9]+(\.[0-9]+)?%`

## `st:ST_FixedPercentage`
Restriction: `base=ST_Percentage pattern=-?((100)\|([0-9][0-9]?))(\.[0-9][0-9]?)?%`

## `st:ST_PositivePercentage`
Restriction: `base=ST_Percentage pattern=[0-9]+(\.[0-9]+)?%`

## `st:ST_PositiveFixedPercentage`
Restriction: `base=ST_Percentage pattern=((100)\|([0-9][0-9]?))(\.[0-9][0-9]?)?%`

## `r:ST_RelationshipId`
Restriction: `base=string`

## `r:@id` : `ST_RelationshipId`

## `r:@embed` : `ST_RelationshipId`

## `r:@link` : `ST_RelationshipId`

## `r:@dm` : `ST_RelationshipId`

## `r:@lo` : `ST_RelationshipId`

## `r:@qs` : `ST_RelationshipId`

## `r:@cs` : `ST_RelationshipId`

## `r:@blip` : `ST_RelationshipId`

## `r:@pict` : `ST_RelationshipId`

## `r:@href` : `ST_RelationshipId`

## `r:@topLeft` : `ST_RelationshipId`

## `r:@topRight` : `ST_RelationshipId`

## `r:@bottomLeft` : `ST_RelationshipId`

## `r:@bottomRight` : `ST_RelationshipId`
