# REVIEW: Change tracking and revision info

Generated. Tier 1M; render -; edit 9; level Retain. 408 rows.


## `x:powerpoint/2013/main/command:ST_CreationId`
Restriction: `base=unsignedInt`

## `x:powerpoint/2013/main/command:CT_DocumentMoniker`

## `x:powerpoint/2013/main/command:CT_SlideMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@cId` | `ST_CreationId` | optional |  |  |
| attribute | `@sldId` | `ST_SlideId` | required |  |  |

## `x:powerpoint/2013/main/command:CT_MainMasterMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@cId` | `ST_CreationId` | optional |  |  |
| attribute | `@sldId` | `ST_SlideMasterId` | required |  |  |

## `x:powerpoint/2013/main/command:CT_SlideLayoutMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@cId` | `ST_CreationId` | optional |  |  |
| attribute | `@sldId` | `ST_SlideLayoutId` | required |  |  |

## `x:powerpoint/2013/main/command:CT_CommentMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@authorId` | `unsignedInt` | required |  |  |
| attribute | `@idx` | `ST_Index` | required |  |  |

## `x:powerpoint/2013/main/command:CT_CommentAuthorMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `unsignedInt` | required |  |  |

## `x:powerpoint/2013/main/command:CT_DocumentMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:powerpoint/2013/main/command:CT_SlideBaseMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:powerpoint/2013/main/command:CT_MainMasterMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:powerpoint/2013/main/command:CT_SlideLayoutMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:powerpoint/2013/main/command:CT_SlideMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:powerpoint/2013/main/command:CT_CommentMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:powerpoint/2013/main/command:CT_CommentAuthorMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:powerpoint/2013/main/command:<cmAuthorMkLst>` : `CT_CommentAuthorMonikerList`

## `x:powerpoint/2013/main/command:<cmMkLst>` : `CT_CommentMonikerList`

## `x:powerpoint/2013/main/command:<docMkLst>` : `CT_DocumentMonikerList`

## `x:powerpoint/2013/main/command:<sldBaseMkLst>` : `CT_SlideBaseMonikerList`

## `x:powerpoint/2013/main/command:<sldLayoutMkLst>` : `CT_SlideLayoutMonikerList`

## `x:powerpoint/2013/main/command:<sldMasterMkLst>` : `CT_MainMasterMonikerList`

## `x:powerpoint/2013/main/command:<sldMkLst>` : `CT_SlideMonikerList`

## `x:powerpoint/2013/main/command:ST_CommentChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `replId`

## `x:powerpoint/2013/main/command:ST_CommentChangeBits`
Restriction: `list=ST_CommentChangeBit`

## `x:powerpoint/2013/main/command:CT_CommentChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `cmMkLst` | `CT_CommentMonikerList` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_CommentChangeBits` | required |  |  |

## `x:powerpoint/2013/main/command:ST_CommentAuthorChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `replId`

## `x:powerpoint/2013/main/command:ST_CommentAuthorChangeBits`
Restriction: `list=ST_CommentAuthorChangeBit`

## `x:powerpoint/2013/main/command:CT_CommentAuthorChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `cmAuthorMkLst` | `CT_CommentAuthorMonikerList` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_CommentAuthorChangeBits` | required |  |  |

## `x:powerpoint/2013/main/command:ST_SlideChangeBit`
Restriction: `base=token`
- `addSp`
- `delSp`
- `modSp`
- `spOrd`
- `new`
- `add`
- `del`
- `mod`
- `ord`
- `replId`
- `modTransition`
- `modMedia`
- `setBg`
- `setFolMasterAnim`
- `setFolMasterObjs`
- `modClrScheme`
- `addAnim`
- `delAnim`
- `modAnim`
- `replTag`
- `delTag`
- `setClrOvrMap`
- `delDesignElem`
- `modShow`
- `addCm`
- `delCm`
- `modCm`
- `chgLayout`
- `modNotes`
- `modNotesTx`
- `setSldSyncInfo`
- `newSectionLinks`

## `x:powerpoint/2013/main/command:ST_SlideChangeBits`
Restriction: `list=ST_SlideChangeBit`

## `x:powerpoint/2013/main/command:CT_SlideChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `sldMkLst` | `CT_SlideMonikerList` |  |  |  |
| element | `spChg` | `CT_ShapeChanges` | 0..unbounded |  |  |
| element | `grpChg` | `CT_GroupShapeChanges` | 0..unbounded |  |  |
| element | `graphicFrameChg` | `CT_GraphicFrameChanges` | 0..unbounded |  |  |
| element | `picChg` | `CT_PictureChanges` | 0..unbounded |  |  |
| element | `inkChg` | `CT_InkChanges` | 0..unbounded |  |  |
| element | `cxnChg` | `CT_ConnectorChanges` | 0..unbounded |  |  |
| element | `cmChg` | `CT_CommentChanges` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_SlideChangeBits` | required |  |  |

## `x:powerpoint/2013/main/command:ST_SlideLayoutChangeBit`
Restriction: `base=token`
- `addSp`
- `delSp`
- `modSp`
- `spOrd`
- `new`
- `add`
- `del`
- `mod`
- `ord`
- `replId`
- `modTransition`
- `modMedia`
- `setBg`
- `setFolMasterAnim`
- `setFolMasterObjs`
- `modClrScheme`
- `addAnim`
- `delAnim`
- `modAnim`
- `replTag`
- `delTag`
- `setClrOvrMap`
- `delDesignElem`
- `modShow`

## `x:powerpoint/2013/main/command:ST_SlideLayoutChangeBits`
Restriction: `list=ST_SlideLayoutChangeBit`

## `x:powerpoint/2013/main/command:CT_SlideLayoutChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `sldLayoutMkLst` | `CT_SlideLayoutMonikerList` |  |  |  |
| element | `spChg` | `CT_ShapeChanges` | 0..unbounded |  |  |
| element | `grpChg` | `CT_GroupShapeChanges` | 0..unbounded |  |  |
| element | `graphicFrameChg` | `CT_GraphicFrameChanges` | 0..unbounded |  |  |
| element | `picChg` | `CT_PictureChanges` | 0..unbounded |  |  |
| element | `inkChg` | `CT_InkChanges` | 0..unbounded |  |  |
| element | `cxnChg` | `CT_ConnectorChanges` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_SlideLayoutChangeBits` | required |  |  |

## `x:powerpoint/2013/main/command:ST_MainMasterChangeBit`
Restriction: `base=token`
- `addSp`
- `delSp`
- `modSp`
- `spOrd`
- `new`
- `add`
- `del`
- `mod`
- `ord`
- `replId`
- `modTransition`
- `modMedia`
- `setBg`
- `setFolMasterAnim`
- `setFolMasterObjs`
- `modClrScheme`
- `addAnim`
- `delAnim`
- `modAnim`
- `replTag`
- `delTag`
- `setClrOvrMap`
- `delDesignElem`
- `modShow`
- `addSldLayout`
- `delSldLayout`
- `modSldLayout`
- `sldLayoutOrd`

## `x:powerpoint/2013/main/command:ST_MainMasterChangeBits`
Restriction: `list=ST_MainMasterChangeBit`

## `x:powerpoint/2013/main/command:CT_MainMasterChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `sldMasterMkLst` | `CT_MainMasterMonikerList` |  |  |  |
| element | `spChg` | `CT_ShapeChanges` | 0..unbounded |  |  |
| element | `grpChg` | `CT_GroupShapeChanges` | 0..unbounded |  |  |
| element | `graphicFrameChg` | `CT_GraphicFrameChanges` | 0..unbounded |  |  |
| element | `picChg` | `CT_PictureChanges` | 0..unbounded |  |  |
| element | `inkChg` | `CT_InkChanges` | 0..unbounded |  |  |
| element | `cxnChg` | `CT_ConnectorChanges` | 0..unbounded |  |  |
| element | `sldLayoutChg` | `CT_SlideLayoutChanges` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_MainMasterChangeBits` | required |  |  |

## `x:powerpoint/2013/main/command:ST_DocumentChangeBit`
Restriction: `base=token`
- `undo`
- `redo`
- `ext`
- `custSel`
- `mod`
- `addSld`
- `delSld`
- `modSld`
- `sldOrd`
- `addMainMaster`
- `delMainMaster`
- `modMainMaster`
- `mainMasterOrd`
- `addSection`
- `delSection`
- `modSection`
- `addCmAuthor`
- `delCmAuthor`
- `modCmAuthor`
- `replTag`
- `delTag`
- `addCustShow`
- `delCustShow`
- `modCustShow`
- `modNotesMaster`
- `modHandout`
- `modShowInfo`
- `addOsfTaskPaneApp`
- `delOsfTaskPaneApp`
- `setSldSz`
- `modRtl`
- `modChgInfo`

## `x:powerpoint/2013/main/command:ST_DocumentChangeBits`
Restriction: `list=ST_DocumentChangeBit`

## `x:powerpoint/2013/main/command:CT_DocumentChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `docMkLst` | `CT_DocumentMonikerList` |  |  |  |
| element | `sldChg` | `CT_SlideChanges` | 0..unbounded |  |  |
| element | `sldMasterChg` | `CT_MainMasterChanges` | 0..unbounded |  |  |
| element | `cmAuthorChg` | `CT_CommentAuthorChanges` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_DocumentChangeBits` | required |  |  |

## `x:powerpoint/2013/main/command:CT_DocumentChangesList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `docChg` | `CT_DocumentChanges` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:powerpoint/2013/main/command:CT_ChangesInfo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `docChgLst` | `CT_DocumentChangesList` | 0..unbounded |  |  |

## `x:powerpoint/2013/main/command:<chgInfo>` : `CT_ChangesInfo`

## `x:powerpoint/2015/10/main:ST_ClientID`
Restriction: `base=string`

## `x:powerpoint/2015/10/main:ST_ClientRevisionNumber`
Restriction: `base=unsignedInt`

## `x:powerpoint/2015/10/main:CT_ClientRevision`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_ClientID` | required |  |  |
| attribute | `@v` | `ST_ClientRevisionNumber` | optional | 0 |  |
| attribute | `@vWet` | `ST_ClientRevisionNumber` | optional | 0 |  |
| attribute | `@dt` | `dateTime` | required |  |  |

## `x:powerpoint/2015/10/main:CT_ClientRevisionList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `client` | `CT_ClientRevision` | 0..unbounded |  |  |

## `x:powerpoint/2015/10/main:CT_RevisionInfo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `revLst` | `CT_ClientRevisionList` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:powerpoint/2015/10/main:<revInfo>` : `CT_RevisionInfo`

## `x:drawing/2013/main/command:AG_DrawingElementId`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_DrawingElementId` | required |  |  |
| attribute | `@creationId` | `ST_Guid` | optional |  |  |

## `x:drawing/2013/main/command:CT_ShapeMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_DrawingElementId` | `AG_DrawingElementId` |  |  |  |

## `x:drawing/2013/main/command:CT_GroupShapeMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_DrawingElementId` | `AG_DrawingElementId` |  |  |  |

## `x:drawing/2013/main/command:CT_GraphicFrameMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_DrawingElementId` | `AG_DrawingElementId` |  |  |  |

## `x:drawing/2013/main/command:CT_ConnectorMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_DrawingElementId` | `AG_DrawingElementId` |  |  |  |

## `x:drawing/2013/main/command:CT_PictureMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_DrawingElementId` | `AG_DrawingElementId` |  |  |  |

## `x:drawing/2013/main/command:CT_InkMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_DrawingElementId` | `AG_DrawingElementId` |  |  |  |

## `x:drawing/2013/main/command:CT_TextCharRangeContext`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@len` | `unsignedInt` | optional | 0 |  |
| attribute | `@hash` | `unsignedInt` | required |  |  |

## `x:drawing/2013/main/command:CT_TextCharRangeMoniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `context` | `CT_TextCharRangeContext` | 0..1 |  |  |
| attribute | `@cp` | `int` | required |  |  |
| attribute | `@len` | `int` | optional | 0 |  |

## `x:drawing/2013/main/command:CT_DrawingElementMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:drawing/2013/main/command:<deMkLst>` : `CT_DrawingElementMonikerList`

## `x:drawing/2013/main/command:CT_ShapeMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:drawing/2013/main/command:<spMkLst>` : `CT_ShapeMonikerList`

## `x:drawing/2013/main/command:CT_GroupShapeMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:drawing/2013/main/command:<grpSpMkLst>` : `CT_GroupShapeMonikerList`

## `x:drawing/2013/main/command:CT_GraphicFrameMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:drawing/2013/main/command:<graphicFrameMkLst>` : `CT_GraphicFrameMonikerList`

## `x:drawing/2013/main/command:CT_ConnectorMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:drawing/2013/main/command:<cxnSpMkLst>` : `CT_ConnectorMonikerList`

## `x:drawing/2013/main/command:CT_PictureMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:drawing/2013/main/command:<picMkLst>` : `CT_PictureMonikerList`

## `x:drawing/2013/main/command:CT_InkMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:drawing/2013/main/command:<inkMkLst>` : `CT_InkMonikerList`

## `x:drawing/2013/main/command:CT_TextCharRangeMonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:drawing/2013/main/command:<txMkLst>` : `CT_TextCharRangeMonikerList`

## `x:drawing/2013/main/command:AG_ChangesUser`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@name` | `string` | optional |  |  |
| attribute | `@userId` | `string` | optional |  |  |
| attribute | `@providerId` | `string` | optional |  |  |
| attribute | `@clId` | `string` | optional |  |  |
| attribute | `@email` | `string` | optional |  |  |

## `x:drawing/2013/main/command:AG_ChangesVersion`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dt` | `dateTime` | optional |  |  |
| attribute | `@v` | `unsignedInt` | optional |  |  |
| attribute | `@id` | `ST_Guid` | optional |  |  |

## `x:drawing/2013/main/command:CT_ChangesData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_ChangesUser` | `AG_ChangesUser` |  |  |  |
| attrgroup | `AG_ChangesVersion` | `AG_ChangesVersion` |  |  |  |
| attribute | `@actId` | `int` | optional |  |  |

## `x:drawing/2013/main/command:ST_ShapeChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `ord`
- `topLvl`
- `modVis`
- `replST`
- `delST`
- `replId`
- `modCrop`

## `x:drawing/2013/main/command:ST_ShapeChangesBits`
Restriction: `list=ST_ShapeChangeBit`

## `x:drawing/2013/main/command:CT_ShapeChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `spMkLst` | `CT_ShapeMonikerList` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_ShapeChangesBits` | required |  |  |

## `x:drawing/2013/main/command:ST_GroupShapeChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `ord`
- `topLvl`
- `modVis`
- `replST`
- `delST`
- `replId`

## `x:drawing/2013/main/command:ST_GroupShapeChangesBits`
Restriction: `list=ST_GroupShapeChangeBit`

## `x:drawing/2013/main/command:CT_GroupShapeChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `grpSpMkLst` | `CT_GroupShapeMonikerList` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_GroupShapeChangesBits` | required |  |  |

## `x:drawing/2013/main/command:ST_ConnectorChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `ord`
- `topLvl`
- `modVis`
- `replST`
- `delST`
- `replId`

## `x:drawing/2013/main/command:ST_ConnectorChangesBits`
Restriction: `list=ST_ConnectorChangeBit`

## `x:drawing/2013/main/command:CT_ConnectorChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `cxnSpMkLst` | `CT_ConnectorMonikerList` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_ConnectorChangesBits` | required |  |  |

## `x:drawing/2013/main/command:ST_PictureChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `ord`
- `topLvl`
- `modVis`
- `replST`
- `delST`
- `replId`
- `modCrop`

## `x:drawing/2013/main/command:ST_PictureChangesBits`
Restriction: `list=ST_PictureChangeBit`

## `x:drawing/2013/main/command:CT_PictureChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `picMkLst` | `CT_PictureMonikerList` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_PictureChangesBits` | required |  |  |

## `x:drawing/2013/main/command:ST_InkChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `ord`
- `topLvl`
- `modVis`
- `replST`
- `delST`
- `replId`
- `reco`
- `modStrokes`

## `x:drawing/2013/main/command:ST_InkChangesBits`
Restriction: `list=ST_InkChangeBit`

## `x:drawing/2013/main/command:CT_InkChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `inkMkLst` | `CT_InkMonikerList` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_InkChangesBits` | required |  |  |

## `x:drawing/2013/main/command:ST_GraphicFrameChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `ord`
- `topLvl`
- `modVis`
- `replST`
- `delST`
- `replId`
- `modGraphic`

## `x:drawing/2013/main/command:ST_GraphicFrameChangesBits`
Restriction: `list=ST_GraphicFrameChangeBit`

## `x:drawing/2013/main/command:CT_GraphicFrameChanges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `graphicFrameMkLst` | `CT_GraphicFrameMonikerList` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_GraphicFrameChangesBits` | required |  |  |
