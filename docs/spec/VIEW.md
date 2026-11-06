# VIEW: Saved view state

Generated. Tier 1B; render -; edit 3; level Edit. 81 rows.


## `p:CT_CommonSlideData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bg` | `CT_Background` | 0..1 |  |  |
| element | `spTree` | `CT_GroupShape` |  |  |  |
| element | `custDataLst` | `CT_CustomerDataList` | 0..1 |  |  |
| element | `controls` | `CT_ControlList` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | optional |  |  |

## `p:ST_SplitterBarState`
Restriction: `base=token`
- `minimized`
- `restored`
- `maximized`

## `p:ST_ViewType`
Restriction: `base=token`
- `sldView`
- `sldMasterView`
- `notesView`
- `handoutView`
- `notesMasterView`
- `outlineView`
- `sldSorterView`
- `sldThumbnailView`

## `p:CT_NormalViewPortion`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@sz` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@autoAdjust` | `boolean` | optional | true |  |

## `p:CT_NormalViewProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `restoredLeft` | `CT_NormalViewPortion` |  |  |  |
| element | `restoredTop` | `CT_NormalViewPortion` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@showOutlineIcons` | `boolean` | optional | true |  |
| attribute | `@snapVertSplitter` | `boolean` | optional | false |  |
| attribute | `@vertBarState` | `ST_SplitterBarState` | optional | restored |  |
| attribute | `@horzBarState` | `ST_SplitterBarState` | optional | restored |  |
| attribute | `@preferSingleView` | `boolean` | optional | false |  |

## `p:CT_CommonViewProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `scale` | `CT_Scale2D` |  |  |  |
| element | `origin` | `CT_Point2D` |  |  |  |
| attribute | `@varScale` | `boolean` | optional | false |  |

## `p:CT_NotesTextViewProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cViewPr` | `CT_CommonViewProperties` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `p:CT_OutlineViewSlideEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |
| attribute | `@collapse` | `boolean` | optional | false |  |

## `p:CT_OutlineViewSlideList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sld` | `CT_OutlineViewSlideEntry` | 0..unbounded |  |  |

## `p:CT_OutlineViewProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cViewPr` | `CT_CommonViewProperties` |  |  |  |
| element | `sldLst` | `CT_OutlineViewSlideList` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `p:CT_SlideSorterViewProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cViewPr` | `CT_CommonViewProperties` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@showFormatting` | `boolean` | optional | true |  |

## `p:CT_Guide`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@orient` | `ST_Direction` | optional | vert |  |
| attribute | `@pos` | `ST_Coordinate32` | optional | 0 |  |

## `p:CT_GuideList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `guide` | `CT_Guide` | 0..unbounded |  |  |

## `p:CT_CommonSlideViewProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cViewPr` | `CT_CommonViewProperties` |  |  |  |
| element | `guideLst` | `CT_GuideList` | 0..1 |  |  |
| attribute | `@snapToGrid` | `boolean` | optional | true |  |
| attribute | `@snapToObjects` | `boolean` | optional | false |  |
| attribute | `@showGuides` | `boolean` | optional | false |  |

## `p:CT_SlideViewProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cSldViewPr` | `CT_CommonSlideViewProperties` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `p:CT_NotesViewProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cSldViewPr` | `CT_CommonSlideViewProperties` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `p:CT_ViewProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `normalViewPr` | `CT_NormalViewProperties` | 0..1 |  |  |
| element | `slideViewPr` | `CT_SlideViewProperties` | 0..1 |  |  |
| element | `outlineViewPr` | `CT_OutlineViewProperties` | 0..1 |  |  |
| element | `notesTextViewPr` | `CT_NotesTextViewProperties` | 0..1 |  |  |
| element | `sorterViewPr` | `CT_SlideSorterViewProperties` | 0..1 |  |  |
| element | `notesViewPr` | `CT_NotesViewProperties` | 0..1 |  |  |
| element | `gridSpacing` | `CT_PositiveSize2D` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@lastView` | `ST_ViewType` | optional | sldView |  |
| attribute | `@showComments` | `boolean` | optional | true |  |

## `p:<viewPr>` : `CT_ViewProperties`
