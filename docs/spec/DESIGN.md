# DESIGN: Designer metadata

Generated. Tier 1M; render -; edit -; level Retain. 14 rows.


## `x:powerpoint/2015/main:CT_DesignElement`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional |  |  |

## `x:powerpoint/2015/main:<designElem>` : `CT_DesignElement`

## `x:powerpoint/2020/02/main:CT_DesignerTag`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@name` | `string` | required |  |  |
| attribute | `@val` | `string` | required |  |  |

## `x:powerpoint/2020/02/main:CT_DesignerTagList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `designTag` | `CT_DesignerTag` | 0..unbounded |  |  |

## `x:powerpoint/2020/02/main:<designTagLst>` : `CT_DesignerTagList`

## `x:powerpoint/2020/02/main:CT_DesignerDrawingProps`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `p202:designTagLst` | `designTagLst` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@edtDesignElem` | `boolean` | optional | false |  |

## `x:powerpoint/2020/02/main:<designPr>` : `CT_DesignerDrawingProps`
