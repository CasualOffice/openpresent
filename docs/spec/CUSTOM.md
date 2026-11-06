# CUSTOM: Tags, customer data, custom XML, app data

Generated. Tier 1M; render -; edit -; level Retain. 53 rows.


## `p:CT_CustomerData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_TagsData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `p:CT_CustomerDataList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `custData` | `CT_CustomerData` | 0..unbounded |  |  |
| element | `tags` | `CT_TagsData` | 0..1 |  |  |

## `p:CT_StringTag`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@name` | `string` | required |  |  |
| attribute | `@val` | `string` | required |  |  |

## `p:CT_TagList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tag` | `CT_StringTag` | 0..unbounded |  |  |

## `p:<tagLst>` : `CT_TagList`

## `ds:CT_DatastoreSchemaRef`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@uri` | `string` | required |  |  |

## `ds:CT_DatastoreSchemaRefs`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `schemaRef` | `CT_DatastoreSchemaRef` | 0..unbounded |  |  |

## `ds:CT_DatastoreItem`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `schemaRefs` | `CT_DatastoreSchemaRefs` | 0..1 |  |  |
| attribute | `@itemID` | `ST_Guid` | required |  |  |

## `ds:<datastoreItem>` : `CT_DatastoreItem`

## `sl:CT_Schema`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@uri` | `string` | optional |  |  |
| attribute | `@manifestLocation` | `string` | optional |  |  |
| attribute | `@schemaLocation` | `string` | optional |  |  |
| attribute | `@schemaLanguage` | `token` | optional |  |  |

## `sl:CT_SchemaLibrary`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `schema` | `CT_Schema` | 0..unbounded |  |  |

## `sl:<schemaLibrary>` : `CT_SchemaLibrary`

## `x:drawing/2021/scriptlink:CT_ScriptLink`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@val` | `string` | optional |  |  |

## `x:drawing/2021/scriptlink:<scriptLink>` : `CT_ScriptLink`

## `x:drawing/2021/livefeed:CT_BackgroundNormalProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2021/livefeed:CT_BackgroundRemovedProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2021/livefeed:CT_BackgroundBlurProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2021/livefeed:CT_BackgroundCustomProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2021/livefeed:EG_LiveFeedBackgrounds`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `Normal` | `CT_BackgroundNormalProperties` | 0..1 |  |  |
| element | `Removed` | `CT_BackgroundRemovedProperties` | 0..1 |  |  |
| element | `Blur` | `CT_BackgroundBlurProperties` | 0..1 |  |  |
| element | `Custom` | `CT_BackgroundCustomProperties` | 0..1 |  |  |

## `x:drawing/2021/livefeed:CT_LiveFeedBackgroundProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_LiveFeedBackgrounds` | `EG_LiveFeedBackgrounds` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2021/livefeed:CT_LiveFeedProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `backgroundProps` | `CT_LiveFeedBackgroundProperties` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2021/livefeed:<liveFeedProps>` : `CT_LiveFeedProperties`
