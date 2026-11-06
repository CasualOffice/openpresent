# MEDIA: Audio and video

Generated. Tier 1H; render 2E; edit 6; level Edit. 47 rows.


## `a:CT_AudioFile`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@r:link` | `link` | required |  |  |
| attribute | `@contentType` | `string` | optional |  |  |

## `a:CT_VideoFile`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@r:link` | `link` | required |  |  |
| attribute | `@contentType` | `string` | optional |  |  |

## `a:CT_QuickTimeFile`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@r:link` | `link` | required |  |  |

## `a:CT_AudioCDTime`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@track` | `unsignedByte` | required |  |  |
| attribute | `@time` | `unsignedInt` | optional | 0 |  |

## `a:CT_AudioCD`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `st` | `CT_AudioCDTime` |  |  |  |
| element | `end` | `CT_AudioCDTime` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:EG_Media`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `audioCd` | `CT_AudioCD` | choice |  |  |
| element | `wavAudioFile` | `CT_EmbeddedWAVAudioFile` | choice |  |  |
| element | `audioFile` | `CT_AudioFile` | choice |  |  |
| element | `videoFile` | `CT_VideoFile` | choice |  |  |
| element | `quickTimeFile` | `CT_QuickTimeFile` | choice |  |  |

## `a:<videoFile>` : `CT_VideoFile`

## `a:CT_EmbeddedWAVAudioFile`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:embed` | `embed` | required |  |  |
| attribute | `@name` | `string` | optional |  |  |

## `x:powerpoint/2017/3/main:ST_DisplayLocation`
Restriction: `base=string`
- `media`
- `slide`

## `x:powerpoint/2017/3/main:CT_Track`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_Guid` | required |  |  |
| attribute | `@label` | `string` | required |  |  |
| attribute | `@lang` | `ST_TextLanguageID` | optional |  |  |
| attrgroup | `AG_Blob` | `AG_Blob` |  |  |  |

## `x:powerpoint/2017/3/main:CT_TrackList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `track` | `CT_Track` | 0..unbounded |  |  |

## `x:powerpoint/2017/3/main:CT_TracksInfo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `trackLst` | `CT_TrackList` | 0..1 |  |  |
| attribute | `@displayLoc` | `ST_DisplayLocation` | required |  |  |

## `x:powerpoint/2017/3/main:<tracksInfo>` : `CT_TracksInfo`

## `x:word/2020/oembed:CT_OEmbed`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@oEmbedUrl` | `string` | required |  |  |
| attribute | `@mediaType` | `string` | required |  |  |
| attribute | `@picLocksAutoForOEmbed` | `boolean` | optional | false |  |

## `x:word/2020/oembed:<oembed>` : `CT_OEmbed`
