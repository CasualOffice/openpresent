# SLIDE: Slides, layouts, masters, notes, handouts

Generated. Tier 1D; render 2A; edit 4; level Edit. 199 rows.


## `p:CT_SlideRelationshipListEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_SlideRelationshipList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sld` | `CT_SlideRelationshipListEntry` | 0..unbounded |  |  |

## `p:ST_SlideId`
Restriction: `base=unsignedInt minInclusive=256 maxExclusive=2147483648`

## `p:CT_SlideIdListEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@id` | `ST_SlideId` | required |  |  |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_SlideIdList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldId` | `CT_SlideIdListEntry` | 0..unbounded |  |  |

## `p:ST_SlideMasterId`
Restriction: `base=unsignedInt minInclusive=2147483648`

## `p:CT_SlideMasterIdListEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@id` | `ST_SlideMasterId` | optional |  |  |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_SlideMasterIdList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldMasterId` | `CT_SlideMasterIdListEntry` | 0..unbounded |  |  |

## `p:CT_NotesMasterIdListEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_NotesMasterIdList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `notesMasterId` | `CT_NotesMasterIdListEntry` | 0..1 |  |  |

## `p:CT_HandoutMasterIdListEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_HandoutMasterIdList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `handoutMasterId` | `CT_HandoutMasterIdListEntry` | 0..1 |  |  |

## `p:CT_SlideSize`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@cx` | `ST_SlideSizeCoordinate` | required |  |  |
| attribute | `@cy` | `ST_SlideSizeCoordinate` | required |  |  |
| attribute | `@type` | `ST_SlideSizeType` | optional | custom |  |

## `p:CT_HeaderFooter`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attribute | `@sldNum` | `boolean` | optional | true |  |
| attribute | `@hdr` | `boolean` | optional | true |  |
| attribute | `@ftr` | `boolean` | optional | true |  |
| attribute | `@dt` | `boolean` | optional | true |  |

## `p:ST_PlaceholderType`
Restriction: `base=token`
- `title`
- `body`
- `ctrTitle`
- `subTitle`
- `dt`
- `sldNum`
- `ftr`
- `hdr`
- `obj`
- `chart`
- `tbl`
- `clipArt`
- `dgm`
- `media`
- `sldImg`
- `pic`

## `p:ST_PlaceholderSize`
Restriction: `base=token`
- `full`
- `half`
- `quarter`

## `p:CT_Placeholder`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attribute | `@type` | `ST_PlaceholderType` | optional | obj |  |
| attribute | `@orient` | `ST_Direction` | optional | horz |  |
| attribute | `@sz` | `ST_PlaceholderSize` | optional | full |  |
| attribute | `@idx` | `unsignedInt` | optional | 0 |  |
| attribute | `@hasCustomPrompt` | `boolean` | optional | false |  |

## `p:CT_Rel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `p:EG_TopLevelSlide`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clrMap` | `CT_ColorMapping` |  |  |  |

## `p:EG_ChildSlide`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clrMapOvr` | `CT_ColorMappingOverride` | 0..1 |  |  |

## `p:AG_ChildSlide`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@showMasterSp` | `boolean` | optional | true |  |
| attribute | `@showMasterPhAnim` | `boolean` | optional | true |  |

## `p:CT_BackgroundProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_FillProperties` | `EG_FillProperties` |  |  |  |
| group | `EG_EffectProperties` | `EG_EffectProperties` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@shadeToTitle` | `boolean` | optional | false |  |

## `p:EG_Background`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bgPr` | `CT_BackgroundProperties` | choice |  |  |
| element | `bgRef` | `CT_StyleMatrixReference` | choice |  |  |

## `p:CT_Background`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_Background` | `EG_Background` |  |  |  |
| attribute | `@bwMode` | `ST_BlackWhiteMode` | optional | white |  |

## `p:CT_Slide`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cSld` | `CT_CommonSlideData` |  |  |  |
| group | `EG_ChildSlide` | `EG_ChildSlide` | 0..1 |  |  |
| element | `transition` | `CT_SlideTransition` | 0..1 |  |  |
| element | `timing` | `CT_SlideTiming` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attrgroup | `AG_ChildSlide` | `AG_ChildSlide` |  |  |  |
| attribute | `@show` | `boolean` | optional | true |  |

## `p:<sld>` : `CT_Slide`

## `p:ST_SlideLayoutType`
Restriction: `base=token`
- `title`
- `tx`
- `twoColTx`
- `tbl`
- `txAndChart`
- `chartAndTx`
- `dgm`
- `chart`
- `txAndClipArt`
- `clipArtAndTx`
- `titleOnly`
- `blank`
- `txAndObj`
- `objAndTx`
- `objOnly`
- `obj`
- `txAndMedia`
- `mediaAndTx`
- `objOverTx`
- `txOverObj`
- `txAndTwoObj`
- `twoObjAndTx`
- `twoObjOverTx`
- `fourObj`
- `vertTx`
- `clipArtAndVertTx`
- `vertTitleAndTx`
- `vertTitleAndTxOverChart`
- `twoObj`
- `objAndTwoObj`
- `twoObjAndObj`
- `cust`
- `secHead`
- `twoTxTwoObj`
- `objTx`
- `picTx`

## `p:CT_SlideLayout`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cSld` | `CT_CommonSlideData` |  |  |  |
| group | `EG_ChildSlide` | `EG_ChildSlide` | 0..1 |  |  |
| element | `transition` | `CT_SlideTransition` | 0..1 |  |  |
| element | `timing` | `CT_SlideTiming` | 0..1 |  |  |
| element | `hf` | `CT_HeaderFooter` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attrgroup | `AG_ChildSlide` | `AG_ChildSlide` |  |  |  |
| attribute | `@matchingName` | `string` | optional |  |  |
| attribute | `@type` | `ST_SlideLayoutType` | optional | cust |  |
| attribute | `@preserve` | `boolean` | optional | false |  |
| attribute | `@userDrawn` | `boolean` | optional | false |  |

## `p:<sldLayout>` : `CT_SlideLayout`

## `p:CT_SlideMasterTextStyles`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `titleStyle` | `CT_TextListStyle` | 0..1 |  |  |
| element | `bodyStyle` | `CT_TextListStyle` | 0..1 |  |  |
| element | `otherStyle` | `CT_TextListStyle` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `p:ST_SlideLayoutId`
Restriction: `base=unsignedInt minInclusive=2147483648`

## `p:CT_SlideLayoutIdListEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@id` | `ST_SlideLayoutId` | optional |  |  |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_SlideLayoutIdList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldLayoutId` | `CT_SlideLayoutIdListEntry` | 0..unbounded |  |  |

## `p:CT_SlideMaster`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cSld` | `CT_CommonSlideData` |  |  |  |
| group | `EG_TopLevelSlide` | `EG_TopLevelSlide` |  |  |  |
| element | `sldLayoutIdLst` | `CT_SlideLayoutIdList` | 0..1 |  |  |
| element | `transition` | `CT_SlideTransition` | 0..1 |  |  |
| element | `timing` | `CT_SlideTiming` | 0..1 |  |  |
| element | `hf` | `CT_HeaderFooter` | 0..1 |  |  |
| element | `txStyles` | `CT_SlideMasterTextStyles` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attribute | `@preserve` | `boolean` | optional | false |  |

## `p:<sldMaster>` : `CT_SlideMaster`

## `p:CT_HandoutMaster`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cSld` | `CT_CommonSlideData` |  |  |  |
| group | `EG_TopLevelSlide` | `EG_TopLevelSlide` |  |  |  |
| element | `hf` | `CT_HeaderFooter` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |

## `p:<handoutMaster>` : `CT_HandoutMaster`

## `p:CT_NotesMaster`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cSld` | `CT_CommonSlideData` |  |  |  |
| group | `EG_TopLevelSlide` | `EG_TopLevelSlide` |  |  |  |
| element | `hf` | `CT_HeaderFooter` | 0..1 |  |  |
| element | `notesStyle` | `CT_TextListStyle` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |

## `p:<notesMaster>` : `CT_NotesMaster`

## `p:CT_NotesSlide`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cSld` | `CT_CommonSlideData` |  |  |  |
| group | `EG_ChildSlide` | `EG_ChildSlide` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attrgroup | `AG_ChildSlide` | `AG_ChildSlide` |  |  |  |

## `p:<notes>` : `CT_NotesSlide`

## `p:CT_SlideSyncProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@serverSldId` | `string` | required |  |  |
| attribute | `@serverSldModifiedTime` | `dateTime` | required |  |  |
| attribute | `@clientInsertedTime` | `dateTime` | required |  |  |

## `p:<sldSyncPr>` : `CT_SlideSyncProperties`

## `x:powerpoint/2023/02/main:CT_PlaceholderTypeACB`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cameo` | `CT_Empty` | choice |  |  |
| element | `unknown` | `CT_Empty` | choice |  |  |

## `x:powerpoint/2023/02/main:CT_PlaceholderTypeExtension`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `type` | `CT_PlaceholderTypeACB` |  |  |  |

## `x:powerpoint/2023/02/main:<phTypeExt>` : `CT_PlaceholderTypeExtension`
