# OLE: OLE, ActiveX, embedded objects

Generated. Tier 1L; render 2F; edit -; level Retain. 30 rows.


## `p:AG_Ole`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@spid` | `ST_ShapeID` | optional |  |  |
| attribute | `@name` | `string` | optional |  |  |
| attribute | `@showAsIcon` | `boolean` | optional | false |  |
| attribute | `@r:id` | `id` | optional |  |  |
| attribute | `@imgW` | `ST_PositiveCoordinate32` | optional |  |  |
| attribute | `@imgH` | `ST_PositiveCoordinate32` | optional |  |  |

## `p:ST_OleObjectFollowColorScheme`
Restriction: `base=token`
- `none`
- `full`
- `textAndBackground`

## `p:CT_OleObjectEmbed`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@followColorScheme` | `ST_OleObjectFollowColorScheme` | optional | none |  |

## `p:CT_OleObjectLink`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@updateAutomatic` | `boolean` | optional | false |  |

## `p:CT_OleObject`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `embed` | `CT_OleObjectEmbed` | choice |  |  |
| element | `link` | `CT_OleObjectLink` | choice |  |  |
| element | `pic` | `CT_Picture` | 0..1 |  |  |
| attrgroup | `AG_Ole` | `AG_Ole` |  |  |  |
| attribute | `@progId` | `string` | optional |  |  |

## `p:<oleObj>` : `CT_OleObject`

## `p:CT_Control`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| element | `pic` | `CT_Picture` | 0..1 |  |  |
| attrgroup | `AG_Ole` | `AG_Ole` |  |  |  |

## `p:CT_ControlList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `control` | `CT_Control` | 0..unbounded |  |  |
