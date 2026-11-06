# PRES: Presentation part and its properties

Generated. Tier 1B; render 2A; edit 4; level Edit. 216 rows.


## `p:CT_Empty`

## `p:ST_Name`
Restriction: `base=string`

## `p:ST_Index`
Restriction: `base=unsignedInt`

## `p:CT_IndexRange`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@st` | `ST_Index` | required |  |  |
| attribute | `@end` | `ST_Index` | required |  |  |

## `p:CT_CustomShowId`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `unsignedInt` | required |  |  |

## `p:EG_SlideListChoice`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldAll` | `CT_Empty` | choice |  |  |
| element | `sldRg` | `CT_IndexRange` | choice |  |  |
| element | `custShow` | `CT_CustomShowId` | choice |  |  |

## `p:CT_Extension`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  | lax |
| attribute | `@uri` | `token` | required |  |  |

## `p:EG_ExtensionList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ext` | `CT_Extension` | 0..unbounded |  |  |

## `p:CT_ExtensionList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ExtensionList` | `EG_ExtensionList` | 0..1 |  |  |

## `p:CT_ExtensionListModify`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ExtensionList` | `EG_ExtensionList` | 0..1 |  |  |
| attribute | `@mod` | `boolean` | optional | false |  |

## `p:CT_EmbeddedFontDataId`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_EmbeddedFontListEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `font` | `CT_TextFont` |  |  |  |
| element | `regular` | `CT_EmbeddedFontDataId` | 0..1 |  |  |
| element | `bold` | `CT_EmbeddedFontDataId` | 0..1 |  |  |
| element | `italic` | `CT_EmbeddedFontDataId` | 0..1 |  |  |
| element | `boldItalic` | `CT_EmbeddedFontDataId` | 0..1 |  |  |

## `p:CT_EmbeddedFontList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `embeddedFont` | `CT_EmbeddedFontListEntry` | 0..unbounded |  |  |

## `p:CT_SmartTags`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_CustomShow`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldLst` | `CT_SlideRelationshipList` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@name` | `ST_Name` | required |  |  |
| attribute | `@id` | `unsignedInt` | required |  |  |

## `p:CT_CustomShowList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `custShow` | `CT_CustomShow` | 0..unbounded |  |  |

## `p:ST_PhotoAlbumLayout`
Restriction: `base=token`
- `fitToSlide`
- `1pic`
- `2pic`
- `4pic`
- `1picTitle`
- `2picTitle`
- `4picTitle`

## `p:ST_PhotoAlbumFrameShape`
Restriction: `base=token`
- `frameStyle1`
- `frameStyle2`
- `frameStyle3`
- `frameStyle4`
- `frameStyle5`
- `frameStyle6`
- `frameStyle7`

## `p:CT_PhotoAlbum`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@bw` | `boolean` | optional | false |  |
| attribute | `@showCaptions` | `boolean` | optional | false |  |
| attribute | `@layout` | `ST_PhotoAlbumLayout` | optional | fitToSlide |  |
| attribute | `@frame` | `ST_PhotoAlbumFrameShape` | optional | frameStyle1 |  |

## `p:ST_SlideSizeCoordinate`
Restriction: `base=ST_PositiveCoordinate32 minInclusive=914400 maxInclusive=51206400`

## `p:ST_SlideSizeType`
Restriction: `base=token`
- `screen4x3`
- `letter`
- `A4`
- `35mm`
- `overhead`
- `banner`
- `custom`
- `ledger`
- `A3`
- `B4ISO`
- `B5ISO`
- `B4JIS`
- `B5JIS`
- `hagakiCard`
- `screen16x9`
- `screen16x10`

## `p:CT_Kinsoku`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@lang` | `string` | optional |  |  |
| attribute | `@invalStChars` | `string` | required |  |  |
| attribute | `@invalEndChars` | `string` | required |  |  |

## `p:ST_BookmarkIdSeed`
Restriction: `base=unsignedInt minInclusive=1 maxExclusive=2147483648`

## `p:CT_ModifyVerifier`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@algorithmName` | `string` | optional |  |  |
| attribute | `@hashValue` | `base64Binary` | optional |  |  |
| attribute | `@saltValue` | `base64Binary` | optional |  |  |
| attribute | `@spinValue` | `unsignedInt` | optional |  |  |
| attribute | `@cryptProviderType` | `ST_CryptProv` | optional |  |  |
| attribute | `@cryptAlgorithmClass` | `ST_AlgClass` | optional |  |  |
| attribute | `@cryptAlgorithmType` | `ST_AlgType` | optional |  |  |
| attribute | `@cryptAlgorithmSid` | `unsignedInt` | optional |  |  |
| attribute | `@spinCount` | `unsignedInt` | optional |  |  |
| attribute | `@saltData` | `base64Binary` | optional |  |  |
| attribute | `@hashData` | `base64Binary` | optional |  |  |
| attribute | `@cryptProvider` | `string` | optional |  |  |
| attribute | `@algIdExt` | `unsignedInt` | optional |  |  |
| attribute | `@algIdExtSource` | `string` | optional |  |  |
| attribute | `@cryptProviderTypeExt` | `unsignedInt` | optional |  |  |
| attribute | `@cryptProviderTypeExtSource` | `string` | optional |  |  |

## `p:CT_Presentation`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldMasterIdLst` | `CT_SlideMasterIdList` | 0..1 |  |  |
| element | `notesMasterIdLst` | `CT_NotesMasterIdList` | 0..1 |  |  |
| element | `handoutMasterIdLst` | `CT_HandoutMasterIdList` | 0..1 |  |  |
| element | `sldIdLst` | `CT_SlideIdList` | 0..1 |  |  |
| element | `sldSz` | `CT_SlideSize` | 0..1 |  |  |
| element | `notesSz` | `CT_PositiveSize2D` |  |  |  |
| element | `smartTags` | `CT_SmartTags` | 0..1 |  |  |
| element | `embeddedFontLst` | `CT_EmbeddedFontList` | 0..1 |  |  |
| element | `custShowLst` | `CT_CustomShowList` | 0..1 |  |  |
| element | `photoAlbum` | `CT_PhotoAlbum` | 0..1 |  |  |
| element | `custDataLst` | `CT_CustomerDataList` | 0..1 |  |  |
| element | `kinsoku` | `CT_Kinsoku` | 0..1 |  |  |
| element | `defaultTextStyle` | `CT_TextListStyle` | 0..1 |  |  |
| element | `modifyVerifier` | `CT_ModifyVerifier` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@serverZoom` | `ST_Percentage` | optional | 50% |  |
| attribute | `@firstSlideNum` | `int` | optional | 1 |  |
| attribute | `@showSpecialPlsOnTitleSld` | `boolean` | optional | true |  |
| attribute | `@rtl` | `boolean` | optional | false |  |
| attribute | `@removePersonalInfoOnSave` | `boolean` | optional | false |  |
| attribute | `@compatMode` | `boolean` | optional | false |  |
| attribute | `@strictFirstAndLastChars` | `boolean` | optional | true |  |
| attribute | `@embedTrueTypeFonts` | `boolean` | optional | false |  |
| attribute | `@saveSubsetFonts` | `boolean` | optional | false |  |
| attribute | `@autoCompressPictures` | `boolean` | optional | true |  |
| attribute | `@bookmarkIdSeed` | `ST_BookmarkIdSeed` | optional | 1 |  |
| attribute | `@conformance` | `ST_ConformanceClass` | optional |  |  |

## `p:<presentation>` : `CT_Presentation`

## `p:CT_HtmlPublishProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SlideListChoice` | `EG_SlideListChoice` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@showSpeakerNotes` | `boolean` | optional | true |  |
| attribute | `@target` | `string` | optional |  |  |
| attribute | `@title` | `string` | optional |  |  |
| attribute | `@r:id` | `id` | required |  |  |

## `p:ST_WebColorType`
Restriction: `base=token`
- `none`
- `browser`
- `presentationText`
- `presentationAccent`
- `whiteTextOnBlack`
- `blackTextOnWhite`

## `p:ST_WebScreenSize`
Restriction: `base=token`
- `544x376`
- `640x480`
- `720x512`
- `800x600`
- `1024x768`
- `1152x882`
- `1152x900`
- `1280x1024`
- `1600x1200`
- `1800x1400`
- `1920x1200`

## `p:ST_WebEncoding`
Restriction: `base=string`

## `p:CT_WebProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@showAnimation` | `boolean` | optional | false |  |
| attribute | `@resizeGraphics` | `boolean` | optional | true |  |
| attribute | `@allowPng` | `boolean` | optional | false |  |
| attribute | `@relyOnVml` | `boolean` | optional | false |  |
| attribute | `@organizeInFolders` | `boolean` | optional | true |  |
| attribute | `@useLongFilenames` | `boolean` | optional | true |  |
| attribute | `@imgSz` | `ST_WebScreenSize` | optional | 800x600 |  |
| attribute | `@encoding` | `ST_WebEncoding` | optional |  |  |
| attribute | `@clr` | `ST_WebColorType` | optional | whiteTextOnBlack |  |

## `p:ST_PrintWhat`
Restriction: `base=token`
- `slides`
- `handouts1`
- `handouts2`
- `handouts3`
- `handouts4`
- `handouts6`
- `handouts9`
- `notes`
- `outline`

## `p:ST_PrintColorMode`
Restriction: `base=token`
- `bw`
- `gray`
- `clr`

## `p:CT_PrintProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@prnWhat` | `ST_PrintWhat` | optional | slides |  |
| attribute | `@clrMode` | `ST_PrintColorMode` | optional | clr |  |
| attribute | `@hiddenSlides` | `boolean` | optional | false |  |
| attribute | `@scaleToFitPaper` | `boolean` | optional | false |  |
| attribute | `@frameSlides` | `boolean` | optional | false |  |

## `p:CT_ShowInfoBrowse`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@showScrollbar` | `boolean` | optional | true |  |

## `p:CT_ShowInfoKiosk`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@restart` | `unsignedInt` | optional | 300000 |  |

## `p:EG_ShowType`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `present` | `CT_Empty` | choice |  |  |
| element | `browse` | `CT_ShowInfoBrowse` | choice |  |  |
| element | `kiosk` | `CT_ShowInfoKiosk` | choice |  |  |

## `p:CT_ShowProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShowType` | `EG_ShowType` | 0..1 |  |  |
| group | `EG_SlideListChoice` | `EG_SlideListChoice` | 0..1 |  |  |
| element | `penClr` | `CT_Color` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@loop` | `boolean` | optional | false |  |
| attribute | `@showNarration` | `boolean` | optional | false |  |
| attribute | `@showAnimation` | `boolean` | optional | true |  |
| attribute | `@useTimings` | `boolean` | optional | true |  |

## `p:CT_PresentationProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `htmlPubPr` | `CT_HtmlPublishProperties` | 0..1 |  |  |
| element | `webPr` | `CT_WebProperties` | 0..1 |  |  |
| element | `prnPr` | `CT_PrintProperties` | 0..1 |  |  |
| element | `showPr` | `CT_ShowProperties` | 0..1 |  |  |
| element | `clrMru` | `CT_ColorMRU` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `p:<presentationPr>` : `CT_PresentationProperties`
