# COMMENT: Comments, legacy and modern

Generated. Tier 1M; render -; edit 9; level Edit. 155 rows.


## `p:CT_CommentAuthor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@id` | `unsignedInt` | required |  |  |
| attribute | `@name` | `ST_Name` | required |  |  |
| attribute | `@initials` | `ST_Name` | required |  |  |
| attribute | `@lastIdx` | `unsignedInt` | required |  |  |
| attribute | `@clrIdx` | `unsignedInt` | required |  |  |

## `p:CT_CommentAuthorList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cmAuthor` | `CT_CommentAuthor` | 0..unbounded |  |  |

## `p:<cmAuthorLst>` : `CT_CommentAuthorList`

## `p:CT_Comment`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pos` | `CT_Point2D` |  |  |  |
| element | `text` | `string` |  |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attribute | `@authorId` | `unsignedInt` | required |  |  |
| attribute | `@dt` | `dateTime` | optional |  |  |
| attribute | `@idx` | `ST_Index` | required |  |  |

## `p:CT_CommentList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cm` | `CT_Comment` | 0..unbounded |  |  |

## `p:<cmLst>` : `CT_CommentList`

## `x:powerpoint/2018/8/main:ST_AuthorId`
Restriction: `base=ST_Guid`

## `x:powerpoint/2018/8/main:CT_Author`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@id` | `ST_AuthorId` | required |  |  |
| attribute | `@name` | `string` | required |  |  |
| attribute | `@initials` | `string` | optional |  |  |
| attribute | `@userId` | `string` | required |  |  |
| attribute | `@providerId` | `string` | required |  |  |

## `x:powerpoint/2018/8/main:CT_AuthorList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `author` | `CT_Author` | 0..unbounded |  |  |

## `x:powerpoint/2018/8/main:ST_AuthorIdList`
Restriction: `list=ST_AuthorId`

## `x:powerpoint/2018/8/main:<authorLst>` : `CT_AuthorList`

## `x:powerpoint/2018/8/main:ST_CommentStatus`
Restriction: `base=token`
- `active`
- `resolved`
- `closed`

## `x:powerpoint/2018/8/main:ST_CommentId`
Restriction: `base=ST_Guid`

## `x:powerpoint/2018/8/main:EG_CommentProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `txBody` | `CT_TextBody` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:powerpoint/2018/8/main:AG_CommentProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_CommentId` | required |  |  |
| attribute | `@authorId` | `ST_AuthorId` | required |  |  |
| attribute | `@status` | `ST_CommentStatus` | optional | active |  |
| attribute | `@created` | `dateTime` | required |  |  |

## `x:powerpoint/2018/8/main:CT_CommentReply`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_CommentProperties` | `EG_CommentProperties` |  |  |  |
| attrgroup | `AG_CommentProperties` | `AG_CommentProperties` |  |  |  |

## `x:powerpoint/2018/8/main:CT_CommentReplyList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `reply` | `CT_CommentReply` | 0..unbounded |  |  |

## `x:powerpoint/2018/8/main:CT_CommentUnknownAnchor`

## `x:powerpoint/2018/8/main:EG_CommentAnchor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pc:sldMkLst` | `sldMkLst` | choice |  |  |
| element | `ac:deMkLst` | `deMkLst` | 0..unbounded |  |  |
| element | `ac:txMkLst` | `txMkLst` | 0..unbounded |  |  |
| element | `unknownAnchor` | `CT_CommentUnknownAnchor` | choice |  |  |

## `x:powerpoint/2018/8/main:CT_Comment`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_CommentAnchor` | `EG_CommentAnchor` |  |  |  |
| element | `pos` | `CT_Point2D` | 0..1 |  |  |
| element | `replyLst` | `CT_CommentReplyList` | 0..1 |  |  |
| group | `EG_CommentProperties` | `EG_CommentProperties` |  |  |  |
| attrgroup | `AG_CommentProperties` | `AG_CommentProperties` |  |  |  |
| attribute | `@startDate` | `dateTime` | optional |  |  |
| attribute | `@dueDate` | `dateTime` | optional |  |  |
| attribute | `@assignedTo` | `ST_AuthorIdList` | optional |  |  |
| attribute | `@complete` | `ST_PositiveFixedPercentage` | optional | 0% |  |
| attribute | `@title` | `string` | optional |  |  |

## `x:powerpoint/2018/8/main:CT_CommentList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cm` | `CT_Comment` | 0..unbounded |  |  |

## `x:powerpoint/2018/8/main:<cmLst>` : `CT_CommentList`

## `x:powerpoint/2018/8/main:CT_CommentRelationship`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `x:powerpoint/2018/8/main:<commentRel>` : `CT_CommentRelationship`

## `x:powerpoint/2019/9/main/command:CT_CommentV2MonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:powerpoint/2019/9/main/command:CT_CommentReplyV2MonikerList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  |  |

## `x:powerpoint/2019/9/main/command:<cmMkLst>` : `CT_CommentV2MonikerList`

## `x:powerpoint/2019/9/main/command:<cmRplyMkLst>` : `CT_CommentReplyV2MonikerList`

## `x:powerpoint/2019/9/main/command:CT_CommentV2Moniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_Guid` | required |  |  |

## `x:powerpoint/2019/9/main/command:CT_CommentReplyV2Moniker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_Guid` | required |  |  |

## `x:powerpoint/2022/03/main:CT_ReactionInstance`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@time` | `dateTime` | required |  |  |
| attribute | `@authorId` | `ST_AuthorId` | required |  |  |

## `x:powerpoint/2022/03/main:CT_Reaction`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `instance` | `CT_ReactionInstance` | 0..unbounded |  |  |
| attribute | `@type` | `string` | required |  |  |

## `x:powerpoint/2022/03/main:CT_Reactions`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `rxn` | `CT_Reaction` | 0..unbounded |  |  |

## `x:powerpoint/2022/03/main:<reactions>` : `CT_Reactions`

## `x:powerpoint/2022/06/main/command:ST_CommentReplyV2ChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `modRxn`

## `x:powerpoint/2022/06/main/command:ST_CommentReplyV2ChangeBits`
Restriction: `list=ST_CommentReplyV2ChangeBit`

## `x:powerpoint/2022/06/main/command:CT_CommentReplyV2Changes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `pc2:cmRplyMkLst` | `cmRplyMkLst` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_CommentReplyV2ChangeBits` | required |  |  |

## `x:powerpoint/2022/06/main/command:ST_CommentV2ChangeBit`
Restriction: `base=token`
- `add`
- `del`
- `mod`
- `modTsk`
- `modRxn`

## `x:powerpoint/2022/06/main/command:ST_CommentV2ChangeBits`
Restriction: `list=ST_CommentV2ChangeBit`

## `x:powerpoint/2022/06/main/command:CT_CommentV2Changes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chgData` | `CT_ChangesData` | 0..1 |  |  |
| element | `pc2:cmMkLst` | `cmMkLst` |  |  |  |
| element | `cmRplyChg` | `CT_CommentReplyV2Changes` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@chg` | `ST_CommentV2ChangeBits` | required |  |  |

## `x:powerpoint/2022/06/main/command:<cmChg>` : `CT_CommentV2Changes`

## `x:powerpoint/2022/08/main:CT_TaskScheduleEventInfo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@stDt` | `dateTime` | optional |  |  |
| attribute | `@endDt` | `dateTime` | optional |  |  |

## `x:powerpoint/2022/08/main:CT_TaskProgressEventInfo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_PositiveFixedPercentage` | required |  |  |

## `x:powerpoint/2022/08/main:CT_TaskTitleEventInfo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `string` | required |  |  |

## `x:powerpoint/2022/08/main:CT_TaskAssignUnassignUser`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@authorId` | `ST_AuthorId` | required |  |  |

## `x:powerpoint/2022/08/main:CT_TaskUnknownRecord`

## `x:powerpoint/2022/08/main:CT_CommentAnchor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_Guid` | required |  |  |

## `x:powerpoint/2022/08/main:CT_TaskAnchor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `comment` | `CT_CommentAnchor` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:powerpoint/2022/08/main:CT_TaskUndo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_Guid` | required |  |  |

## `x:powerpoint/2022/08/main:CT_TaskHistoryEvent`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `atrbtn` | `CT_TaskAssignUnassignUser` |  |  |  |
| element | `anchr` | `CT_TaskAnchor` | 0..1 |  |  |
| element | `asgn` | `CT_TaskAssignUnassignUser` | choice |  |  |
| element | `add` | `CT_Empty` | 0..1 |  |  |
| element | `title` | `CT_TaskTitleEventInfo` | choice |  |  |
| element | `date` | `CT_TaskScheduleEventInfo` | choice |  |  |
| element | `pcntCmplt` | `CT_TaskProgressEventInfo` | choice |  |  |
| element | `unasgnAll` | `CT_Empty` | 0..1 |  |  |
| element | `undo` | `CT_TaskUndo` | choice |  |  |
| element | `unknown` | `CT_TaskUnknownRecord` | choice |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@time` | `dateTime` | required |  |  |
| attribute | `@id` | `ST_Guid` | required |  |  |

## `x:powerpoint/2022/08/main:CT_TaskHistory`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `event` | `CT_TaskHistoryEvent` | 0..unbounded |  |  |

## `x:powerpoint/2022/08/main:CT_TaskDetails`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `history` | `CT_TaskHistory` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:powerpoint/2022/08/main:<taskDetails>` : `CT_TaskDetails`
