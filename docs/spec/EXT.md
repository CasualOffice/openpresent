# EXT: PowerPoint extensions: sections, zoom, classification

Generated. Tier 1M; render 2F; edit 8; level Edit. 246 rows.


## `x:powerpoint/2010/main:ST_UniversalTimeOffset`
Restriction: `base=string`

## `x:powerpoint/2010/main:ST_TransitionPattern`
Restriction: `base=token`
- `diamond`
- `hexagon`

## `x:powerpoint/2010/main:ST_TransitionCenterDirectionType`
Restriction: `base=token`
- `center`

## `x:powerpoint/2010/main:ST_TransitionShredPattern`
Restriction: `base=token`
- `strip`
- `rectangle`

## `x:powerpoint/2010/main:ST_TransitionCornerAndCenterDirectionType`
Restriction: `union=ST_TransitionCornerDirectionType ST_TransitionCenterDirectionType`

## `x:powerpoint/2010/main:ST_TransitionLeftRightDirectionType`
Restriction: `base=token`
- `l`
- `r`

## `x:powerpoint/2010/main:CT_LeftRightDirectionTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_TransitionLeftRightDirectionType` | optional |  |  |

## `x:powerpoint/2010/main:<vortex>` : `CT_SideDirectionTransition`

## `x:powerpoint/2010/main:<switch>` : `CT_LeftRightDirectionTransition`

## `x:powerpoint/2010/main:<flip>` : `CT_LeftRightDirectionTransition`

## `x:powerpoint/2010/main:CT_RippleTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_TransitionCornerAndCenterDirectionType` | optional | center |  |

## `x:powerpoint/2010/main:<ripple>` : `CT_RippleTransition`

## `x:powerpoint/2010/main:<honeycomb>` : `CT_Empty`

## `x:powerpoint/2010/main:CT_PrismTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_TransitionSideDirectionType` | optional | l |  |
| attribute | `@isContent` | `boolean` | optional | false |  |
| attribute | `@isInverted` | `boolean` | optional | false |  |

## `x:powerpoint/2010/main:<prism>` : `CT_PrismTransition`

## `x:powerpoint/2010/main:<doors>` : `CT_OrientationTransition`

## `x:powerpoint/2010/main:<window>` : `CT_OrientationTransition`

## `x:powerpoint/2010/main:<ferris>` : `CT_LeftRightDirectionTransition`

## `x:powerpoint/2010/main:<gallery>` : `CT_LeftRightDirectionTransition`

## `x:powerpoint/2010/main:<conveyor>` : `CT_LeftRightDirectionTransition`

## `x:powerpoint/2010/main:<pan>` : `CT_SideDirectionTransition`

## `x:powerpoint/2010/main:CT_GlitterTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_TransitionSideDirectionType` | optional | l |  |
| attribute | `@pattern` | `ST_TransitionPattern` | optional | diamond |  |

## `x:powerpoint/2010/main:<glitter>` : `CT_GlitterTransition`

## `x:powerpoint/2010/main:CT_FlyThroughTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_TransitionInOutDirectionType` | optional | in |  |
| attribute | `@hasBounce` | `boolean` | optional | false |  |

## `x:powerpoint/2010/main:<warp>` : `CT_InOutTransition`

## `x:powerpoint/2010/main:<flythrough>` : `CT_FlyThroughTransition`

## `x:powerpoint/2010/main:<flash>` : `CT_Empty`

## `x:powerpoint/2010/main:CT_ShredTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@pattern` | `ST_TransitionShredPattern` | optional | strip |  |
| attribute | `@dir` | `ST_TransitionInOutDirectionType` | optional | in |  |

## `x:powerpoint/2010/main:<shred>` : `CT_ShredTransition`

## `x:powerpoint/2010/main:CT_RevealTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@thruBlk` | `boolean` | optional | false |  |
| attribute | `@dir` | `ST_TransitionLeftRightDirectionType` | optional | l |  |

## `x:powerpoint/2010/main:<reveal>` : `CT_RevealTransition`

## `x:powerpoint/2010/main:CT_MediaBookmarkTarget`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@spid` | `ST_DrawingElementId` | required |  |  |
| attribute | `@bmkName` | `string` | required |  |  |

## `x:powerpoint/2010/main:<wheelReverse>` : `CT_WheelTransition`

## `x:powerpoint/2010/main:@dur` : `ST_UniversalTimeOffset`

## `x:powerpoint/2010/main:<bmkTgt>` : `CT_MediaBookmarkTarget`

## `x:powerpoint/2010/main:@presetBounceEnd` : `ST_PositiveFixedPercentage`

## `x:powerpoint/2010/main:@bounceEnd` : `ST_PositiveFixedPercentage`

## `x:powerpoint/2010/main:CT_ContentPartNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvContentPartPr` | `CT_NonVisualInkContentPartProperties` | 0..1 |  |  |
| element | `nvPr` | `CT_ApplicationNonVisualDrawingProps` |  |  |  |

## `x:powerpoint/2010/main:<nvContentPartPr>` : `CT_ContentPartNonVisual`

## `x:powerpoint/2010/main:@bwMode` : `ST_BlackWhiteMode`

## `x:powerpoint/2010/main:<xfrm>` : `CT_Transform2D`

## `x:powerpoint/2010/main:<extLst>` : `CT_ExtensionListModify`

## `x:powerpoint/2010/main:CT_MediaTrim`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@st` | `ST_UniversalTimeOffset` | optional | 0 |  |
| attribute | `@end` | `ST_UniversalTimeOffset` | optional | 0 |  |

## `x:powerpoint/2010/main:CT_MediaFade`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@in` | `ST_UniversalTimeOffset` | optional | 0 |  |
| attribute | `@out` | `ST_UniversalTimeOffset` | optional | 0 |  |

## `x:powerpoint/2010/main:CT_MediaBookmark`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@name` | `string` | optional |  |  |
| attribute | `@time` | `ST_UniversalTimeOffset` | optional |  |  |

## `x:powerpoint/2010/main:CT_MediaBookmarkList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bmk` | `CT_MediaBookmark` | 0..unbounded |  |  |

## `x:powerpoint/2010/main:CT_Media`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `trim` | `CT_MediaTrim` | 0..1 |  |  |
| element | `fade` | `CT_MediaFade` | 0..1 |  |  |
| element | `bmkLst` | `CT_MediaBookmarkList` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attrgroup | `AG_Blob` | `AG_Blob` |  |  |  |

## `x:powerpoint/2010/main:<media>` : `CT_Media`

## `x:powerpoint/2010/main:CT_SectionSlideIdListEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_SlideId` | required |  |  |

## `x:powerpoint/2010/main:CT_SectionSlideIdList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldId` | `CT_SectionSlideIdListEntry` | 0..unbounded |  |  |

## `x:powerpoint/2010/main:CT_Section`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldIdLst` | `CT_SectionSlideIdList` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | optional |  |  |
| attribute | `@id` | `ST_Guid` | optional |  |  |

## `x:powerpoint/2010/main:CT_SectionList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `section` | `CT_Section` | 1..unbounded |  |  |

## `x:powerpoint/2010/main:<sectionLst>` : `CT_SectionList`

## `x:powerpoint/2010/main:CT_BrowseMode`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@showStatus` | `boolean` | optional | true |  |

## `x:powerpoint/2010/main:<browseMode>` : `CT_BrowseMode`

## `x:powerpoint/2010/main:<laserClr>` : `CT_Color`

## `x:powerpoint/2010/main:CT_DefaultImageDpi`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `unsignedInt` | required |  |  |

## `x:powerpoint/2010/main:<defaultImageDpi>` : `CT_DefaultImageDpi`

## `x:powerpoint/2010/main:CT_DiscardImageEditData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | required |  |  |

## `x:powerpoint/2010/main:<discardImageEditData>` : `CT_DiscardImageEditData`

## `x:powerpoint/2010/main:CT_ShowMediaControls`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | required |  |  |

## `x:powerpoint/2010/main:<showMediaCtrls>` : `CT_ShowMediaControls`

## `x:powerpoint/2010/main:CT_LaserTracePoint`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@t` | `ST_UniversalTimeOffset` | required |  |  |
| attribute | `@x` | `ST_Coordinate` | required |  |  |
| attribute | `@y` | `ST_Coordinate` | required |  |  |

## `x:powerpoint/2010/main:CT_LaserTrace`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tracePt` | `CT_LaserTracePoint` | 0..unbounded |  |  |

## `x:powerpoint/2010/main:CT_LaserTraceList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tracePtLst` | `CT_LaserTrace` | 0..unbounded |  |  |

## `x:powerpoint/2010/main:<laserTraceLst>` : `CT_LaserTraceList`

## `x:powerpoint/2010/main:CT_RandomId`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `unsignedInt` | required |  |  |

## `x:powerpoint/2010/main:<creationId>` : `CT_RandomId`

## `x:powerpoint/2010/main:<modId>` : `CT_RandomId`

## `x:powerpoint/2010/main:CT_TriggerEventRecord`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `ST_TLTriggerEvent` | required |  |  |
| attribute | `@time` | `ST_UniversalTimeOffset` | required |  |  |
| attribute | `@objId` | `ST_DrawingElementId` | required |  |  |

## `x:powerpoint/2010/main:CT_NullEventRecord`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@time` | `ST_UniversalTimeOffset` | required |  |  |
| attribute | `@objId` | `ST_DrawingElementId` | required |  |  |

## `x:powerpoint/2010/main:CT_MediaPlaybackEventRecord`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@time` | `ST_UniversalTimeOffset` | required |  |  |
| attribute | `@objId` | `ST_DrawingElementId` | required |  |  |

## `x:powerpoint/2010/main:CT_MediaSeekEventRecord`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@time` | `ST_UniversalTimeOffset` | required |  |  |
| attribute | `@objId` | `ST_DrawingElementId` | required |  |  |
| attribute | `@seek` | `ST_UniversalTimeOffset` | required |  |  |

## `x:powerpoint/2010/main:CT_ShowEventRecordList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `triggerEvt` | `CT_TriggerEventRecord` | choice |  |  |
| element | `playEvt` | `CT_MediaPlaybackEventRecord` | choice |  |  |
| element | `stopEvt` | `CT_MediaPlaybackEventRecord` | choice |  |  |
| element | `pauseEvt` | `CT_MediaPlaybackEventRecord` | choice |  |  |
| element | `resumeEvt` | `CT_MediaPlaybackEventRecord` | choice |  |  |
| element | `seekEvt` | `CT_MediaSeekEventRecord` | choice |  |  |
| element | `nullEvt` | `CT_NullEventRecord` | choice |  |  |

## `x:powerpoint/2010/main:<showEvtLst>` : `CT_ShowEventRecordList`

## `x:powerpoint/2012/main:CT_PresetTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@prst` | `string` | optional |  |  |
| attribute | `@invX` | `boolean` | optional | false |  |
| attribute | `@invY` | `boolean` | optional | false |  |

## `x:powerpoint/2012/main:<prstTrans>` : `CT_PresetTransition`

## `x:powerpoint/2012/main:CT_PresenceInfo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@userId` | `string` | required |  |  |
| attribute | `@providerId` | `string` | required |  |  |

## `x:powerpoint/2012/main:CT_ParentCommentIdentifier`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@authorId` | `unsignedInt` | optional |  |  |
| attribute | `@idx` | `unsignedInt` | optional |  |  |

## `x:powerpoint/2012/main:CT_CommentThreading`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `parentCm` | `CT_ParentCommentIdentifier` | 0..1 |  |  |
| attribute | `@timeZoneBias` | `int` | optional |  |  |

## `x:powerpoint/2012/main:<presenceInfo>` : `CT_PresenceInfo`

## `x:powerpoint/2012/main:<threadingInfo>` : `CT_CommentThreading`

## `x:powerpoint/2012/main:CT_ExtendedGuide`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clr` | `CT_Color` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@id` | `unsignedInt` | required |  |  |
| attribute | `@name` | `string` | optional |  |  |
| attribute | `@orient` | `ST_Direction` | optional | vert |  |
| attribute | `@pos` | `ST_Coordinate32` | optional | 0 |  |
| attribute | `@userDrawn` | `boolean` | optional | false |  |

## `x:powerpoint/2012/main:CT_ExtendedGuideList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `guide` | `CT_ExtendedGuide` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:powerpoint/2012/main:<sldGuideLst>` : `CT_ExtendedGuideList`

## `x:powerpoint/2012/main:<notesGuideLst>` : `CT_ExtendedGuideList`

## `x:powerpoint/2012/main:CT_ChartTrackingRefBased`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | required |  |  |

## `x:powerpoint/2012/main:<chartTrackingRefBased>` : `CT_ChartTrackingRefBased`

## `x:powerpoint/2012/main:CT_IsNarration`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | false |  |

## `x:powerpoint/2012/main:<isNarration>` : `CT_IsNarration`

## `x:powerpoint/2016/6/main:ST_ZoomObjectImageType`
Restriction: `base=token`
- `preview`
- `cover`

## `x:powerpoint/2016/6/main:CT_ZoomObjectProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `blipFill` | `CT_BlipFillProperties` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| attribute | `@id` | `ST_Guid` | required |  |  |
| attribute | `@returnToParent` | `boolean` | optional | true |  |
| attribute | `@imageType` | `ST_ZoomObjectImageType` | optional | preview |  |
| attribute | `@transitionDur` | `ST_UniversalTimeOffset` | optional |  |  |
| attribute | `@showBg` | `boolean` | optional | true |  |

## `x:powerpoint/2016/sectionzoom:CT_SectionZoomObject`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `zmPr` | `CT_ZoomObjectProperties` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@sectionId` | `ST_Guid` | required |  |  |

## `x:powerpoint/2016/sectionzoom:CT_SectionZoom`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sectionZmObj` | `CT_SectionZoomObject` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:powerpoint/2016/sectionzoom:<sectionZm>` : `CT_SectionZoom`

## `x:powerpoint/2016/slidezoom:CT_SlideZoomObject`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `zmPr` | `CT_ZoomObjectProperties` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@sldId` | `ST_SlideId` | required |  |  |
| attribute | `@cId` | `unsignedInt` | optional |  |  |

## `x:powerpoint/2016/slidezoom:CT_SlideZoom`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldZmObj` | `CT_SlideZoomObject` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:powerpoint/2016/slidezoom:<sldZm>` : `CT_SlideZoom`

## `x:powerpoint/2016/summaryzoom:CT_SummaryZoomObject`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `zmPr` | `CT_ZoomObjectProperties` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@sectionId` | `ST_Guid` | required |  |  |
| attribute | `@title` | `string` | optional |  |  |
| attribute | `@descr` | `string` | optional |  |  |
| attribute | `@offsetFactorX` | `ST_Percentage` | optional | 0 |  |
| attribute | `@offsetFactorY` | `ST_Percentage` | optional | 0 |  |
| attribute | `@scaleFactorX` | `ST_Percentage` | optional | 100000 |  |
| attribute | `@scaleFactorY` | `ST_Percentage` | optional | 100000 |  |

## `x:powerpoint/2016/summaryzoom:CT_GridLayout`

## `x:powerpoint/2016/summaryzoom:CT_FixedLayout`

## `x:powerpoint/2016/summaryzoom:CT_SummaryZoom`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `summaryZmObj` | `CT_SummaryZoomObject` | 0..unbounded |  |  |
| element | `gridLayout` | `CT_GridLayout` | choice |  |  |
| element | `fixedLayout` | `CT_FixedLayout` | choice |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:powerpoint/2016/summaryzoom:<summaryZm>` : `CT_SummaryZoom`

## `x:powerpoint/2017/10/main:CT_ReadonlyRecommended`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | required |  |  |

## `x:powerpoint/2017/10/main:<readonlyRecommended>` : `CT_ReadonlyRecommended`

## `x:powerpoint/2018/4/main:ST_ClassificationOutcomeType`
Restriction: `base=token`
- `none`
- `hdr`
- `ftr`
- `watermark`

## `x:powerpoint/2018/4/main:CT_ClassificationOutcome`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_ClassificationOutcomeType` | optional |  |  |

## `x:powerpoint/2018/4/main:<classification>` : `CT_ClassificationOutcome`

## `x:drawing/2020/classificationShape:ST_ClassificationOutcomeType`
Restriction: `base=token`
- `none`
- `hdr`
- `ftr`
- `watermark`

## `x:drawing/2020/classificationShape:CT_ClassificationOutcome`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@classificationOutcomeType` | `ST_ClassificationOutcomeType` | optional |  |  |

## `x:drawing/2020/classificationShape:<classification>` : `CT_ClassificationOutcome`
