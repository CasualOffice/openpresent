# PKG: Package, relationships, properties

Generated. Tier 1A; render -; edit 10; level Edit. 285 rows.


## `ep:<Properties>` : `CT_Properties`

## `ep:CT_Properties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `Template` | `string` | 0..1 |  |  |
| element | `Manager` | `string` | 0..1 |  |  |
| element | `Company` | `string` | 0..1 |  |  |
| element | `Pages` | `int` | 0..1 |  |  |
| element | `Words` | `int` | 0..1 |  |  |
| element | `Characters` | `int` | 0..1 |  |  |
| element | `PresentationFormat` | `string` | 0..1 |  |  |
| element | `Lines` | `int` | 0..1 |  |  |
| element | `Paragraphs` | `int` | 0..1 |  |  |
| element | `Slides` | `int` | 0..1 |  |  |
| element | `Notes` | `int` | 0..1 |  |  |
| element | `TotalTime` | `int` | 0..1 |  |  |
| element | `HiddenSlides` | `int` | 0..1 |  |  |
| element | `MMClips` | `int` | 0..1 |  |  |
| element | `ScaleCrop` | `boolean` | 0..1 |  |  |
| element | `HeadingPairs` | `CT_VectorVariant` | 0..1 |  |  |
| element | `TitlesOfParts` | `CT_VectorLpstr` | 0..1 |  |  |
| element | `LinksUpToDate` | `boolean` | 0..1 |  |  |
| element | `CharactersWithSpaces` | `int` | 0..1 |  |  |
| element | `SharedDoc` | `boolean` | 0..1 |  |  |
| element | `HyperlinkBase` | `string` | 0..1 |  |  |
| element | `HLinks` | `CT_VectorVariant` | 0..1 |  |  |
| element | `HyperlinksChanged` | `boolean` | 0..1 |  |  |
| element | `DigSig` | `CT_DigSigBlob` | 0..1 |  |  |
| element | `Application` | `string` | 0..1 |  |  |
| element | `AppVersion` | `string` | 0..1 |  |  |
| element | `DocSecurity` | `int` | 0..1 |  |  |

## `ep:CT_VectorVariant`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `vt:vector` | `vector` |  |  |  |

## `ep:CT_VectorLpstr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `vt:vector` | `vector` |  |  |  |

## `ep:CT_DigSigBlob`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `vt:blob` | `blob` |  |  |  |

## `cp:<Properties>` : `CT_Properties`

## `cp:CT_Properties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `property` | `CT_Property` | 0..unbounded |  |  |

## `cp:CT_Property`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `vt:vector` | `vector` | choice |  |  |
| element | `vt:array` | `array` | choice |  |  |
| element | `vt:blob` | `blob` | choice |  |  |
| element | `vt:oblob` | `oblob` | choice |  |  |
| element | `vt:empty` | `empty` | choice |  |  |
| element | `vt:null` | `null` | choice |  |  |
| element | `vt:i1` | `i1` | choice |  |  |
| element | `vt:i2` | `i2` | choice |  |  |
| element | `vt:i4` | `i4` | choice |  |  |
| element | `vt:i8` | `i8` | choice |  |  |
| element | `vt:int` | `int` | choice |  |  |
| element | `vt:ui1` | `ui1` | choice |  |  |
| element | `vt:ui2` | `ui2` | choice |  |  |
| element | `vt:ui4` | `ui4` | choice |  |  |
| element | `vt:ui8` | `ui8` | choice |  |  |
| element | `vt:uint` | `uint` | choice |  |  |
| element | `vt:r4` | `r4` | choice |  |  |
| element | `vt:r8` | `r8` | choice |  |  |
| element | `vt:decimal` | `decimal` | choice |  |  |
| element | `vt:lpstr` | `lpstr` | choice |  |  |
| element | `vt:lpwstr` | `lpwstr` | choice |  |  |
| element | `vt:bstr` | `bstr` | choice |  |  |
| element | `vt:date` | `date` | choice |  |  |
| element | `vt:filetime` | `filetime` | choice |  |  |
| element | `vt:bool` | `bool` | choice |  |  |
| element | `vt:cy` | `cy` | choice |  |  |
| element | `vt:error` | `error` | choice |  |  |
| element | `vt:stream` | `stream` | choice |  |  |
| element | `vt:ostream` | `ostream` | choice |  |  |
| element | `vt:storage` | `storage` | choice |  |  |
| element | `vt:ostorage` | `ostorage` | choice |  |  |
| element | `vt:vstream` | `vstream` | choice |  |  |
| element | `vt:clsid` | `clsid` | choice |  |  |
| attribute | `@fmtid` | `ST_Guid` | required |  |  |
| attribute | `@pid` | `int` | required |  |  |
| attribute | `@name` | `string` | optional |  |  |
| attribute | `@linkTarget` | `string` | optional |  |  |

## `vt:ST_VectorBaseType`
Restriction: `base=string`
- `variant`
- `i1`
- `i2`
- `i4`
- `i8`
- `ui1`
- `ui2`
- `ui4`
- `ui8`
- `r4`
- `r8`
- `lpstr`
- `lpwstr`
- `bstr`
- `date`
- `filetime`
- `bool`
- `cy`
- `error`
- `clsid`

## `vt:ST_ArrayBaseType`
Restriction: `base=string`
- `variant`
- `i1`
- `i2`
- `i4`
- `int`
- `ui1`
- `ui2`
- `ui4`
- `uint`
- `r4`
- `r8`
- `decimal`
- `bstr`
- `date`
- `bool`
- `cy`
- `error`

## `vt:ST_Cy`
Restriction: `base=string pattern=\s*[0-9]*\.[0-9]{4}\s*`

## `vt:ST_Error`
Restriction: `base=string pattern=\s*0x[0-9A-Za-z]{8}\s*`

## `vt:CT_Empty`

## `vt:CT_Null`

## `vt:CT_Vector`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `variant` | `variant` | choice |  |  |
| element | `i1` | `i1` | choice |  |  |
| element | `i2` | `i2` | choice |  |  |
| element | `i4` | `i4` | choice |  |  |
| element | `i8` | `i8` | choice |  |  |
| element | `ui1` | `ui1` | choice |  |  |
| element | `ui2` | `ui2` | choice |  |  |
| element | `ui4` | `ui4` | choice |  |  |
| element | `ui8` | `ui8` | choice |  |  |
| element | `r4` | `r4` | choice |  |  |
| element | `r8` | `r8` | choice |  |  |
| element | `lpstr` | `lpstr` | choice |  |  |
| element | `lpwstr` | `lpwstr` | choice |  |  |
| element | `bstr` | `bstr` | choice |  |  |
| element | `date` | `date` | choice |  |  |
| element | `filetime` | `filetime` | choice |  |  |
| element | `bool` | `bool` | choice |  |  |
| element | `cy` | `cy` | choice |  |  |
| element | `error` | `error` | choice |  |  |
| element | `clsid` | `clsid` | choice |  |  |
| attribute | `@baseType` | `ST_VectorBaseType` | required |  |  |
| attribute | `@size` | `unsignedInt` | required |  |  |

## `vt:CT_Array`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `variant` | `variant` | choice |  |  |
| element | `i1` | `i1` | choice |  |  |
| element | `i2` | `i2` | choice |  |  |
| element | `i4` | `i4` | choice |  |  |
| element | `int` | `int` | choice |  |  |
| element | `ui1` | `ui1` | choice |  |  |
| element | `ui2` | `ui2` | choice |  |  |
| element | `ui4` | `ui4` | choice |  |  |
| element | `uint` | `uint` | choice |  |  |
| element | `r4` | `r4` | choice |  |  |
| element | `r8` | `r8` | choice |  |  |
| element | `decimal` | `decimal` | choice |  |  |
| element | `bstr` | `bstr` | choice |  |  |
| element | `date` | `date` | choice |  |  |
| element | `bool` | `bool` | choice |  |  |
| element | `error` | `error` | choice |  |  |
| element | `cy` | `cy` | choice |  |  |
| attribute | `@lBounds` | `int` | required |  |  |
| attribute | `@uBounds` | `int` | required |  |  |
| attribute | `@baseType` | `ST_ArrayBaseType` | required |  |  |

## `vt:CT_Variant`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `variant` | `variant` | choice |  |  |
| element | `vector` | `vector` | choice |  |  |
| element | `array` | `array` | choice |  |  |
| element | `blob` | `blob` | choice |  |  |
| element | `oblob` | `oblob` | choice |  |  |
| element | `empty` | `empty` | choice |  |  |
| element | `null` | `null` | choice |  |  |
| element | `i1` | `i1` | choice |  |  |
| element | `i2` | `i2` | choice |  |  |
| element | `i4` | `i4` | choice |  |  |
| element | `i8` | `i8` | choice |  |  |
| element | `int` | `int` | choice |  |  |
| element | `ui1` | `ui1` | choice |  |  |
| element | `ui2` | `ui2` | choice |  |  |
| element | `ui4` | `ui4` | choice |  |  |
| element | `ui8` | `ui8` | choice |  |  |
| element | `uint` | `uint` | choice |  |  |
| element | `r4` | `r4` | choice |  |  |
| element | `r8` | `r8` | choice |  |  |
| element | `decimal` | `decimal` | choice |  |  |
| element | `lpstr` | `lpstr` | choice |  |  |
| element | `lpwstr` | `lpwstr` | choice |  |  |
| element | `bstr` | `bstr` | choice |  |  |
| element | `date` | `date` | choice |  |  |
| element | `filetime` | `filetime` | choice |  |  |
| element | `bool` | `bool` | choice |  |  |
| element | `cy` | `cy` | choice |  |  |
| element | `error` | `error` | choice |  |  |
| element | `stream` | `stream` | choice |  |  |
| element | `ostream` | `ostream` | choice |  |  |
| element | `storage` | `storage` | choice |  |  |
| element | `ostorage` | `ostorage` | choice |  |  |
| element | `vstream` | `vstream` | choice |  |  |
| element | `clsid` | `clsid` | choice |  |  |

## `vt:CT_Vstream`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `base64Binary` | `base64Binary` |  |  |  |
| attribute | `@version` | `ST_Guid` | optional |  |  |

## `vt:<variant>` : `CT_Variant`

## `vt:<vector>` : `CT_Vector`

## `vt:<array>` : `CT_Array`

## `vt:<blob>` : `base64Binary`

## `vt:<oblob>` : `base64Binary`

## `vt:<empty>` : `CT_Empty`

## `vt:<null>` : `CT_Null`

## `vt:<i1>` : `byte`

## `vt:<i2>` : `short`

## `vt:<i4>` : `int`

## `vt:<i8>` : `long`

## `vt:<int>` : `int`

## `vt:<ui1>` : `unsignedByte`

## `vt:<ui2>` : `unsignedShort`

## `vt:<ui4>` : `unsignedInt`

## `vt:<ui8>` : `unsignedLong`

## `vt:<uint>` : `unsignedInt`

## `vt:<r4>` : `float`

## `vt:<r8>` : `double`

## `vt:<decimal>` : `decimal`

## `vt:<lpstr>` : `string`

## `vt:<lpwstr>` : `string`

## `vt:<bstr>` : `string`

## `vt:<date>` : `dateTime`

## `vt:<filetime>` : `dateTime`

## `vt:<bool>` : `boolean`

## `vt:<cy>` : `ST_Cy`

## `vt:<error>` : `ST_Error`

## `vt:<stream>` : `base64Binary`

## `vt:<ostream>` : `base64Binary`

## `vt:<storage>` : `base64Binary`

## `vt:<ostorage>` : `base64Binary`

## `vt:<vstream>` : `CT_Vstream`

## `vt:<clsid>` : `ST_Guid`

## `ct:<Types>` : `CT_Types`

## `ct:<Default>` : `CT_Default`

## `ct:<Override>` : `CT_Override`

## `ct:CT_Types`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `Default` | `Default` | choice |  |  |
| element | `Override` | `Override` | choice |  |  |

## `ct:CT_Default`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@Extension` | `ST_Extension` | required |  |  |
| attribute | `@ContentType` | `ST_ContentType` | required |  |  |

## `ct:CT_Override`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@ContentType` | `ST_ContentType` | required |  |  |
| attribute | `@PartName` | `anyURI` | required |  |  |

## `ct:ST_ContentType`
Restriction: `base=string pattern=(((([\p{IsBasicLatin}-[\p{Cc}\(\)<>@,;:\\"/\[\]\?=\{\}\s\t]])+))/((([\p{IsBasicLatin}-[\p{Cc}\(\)<>@,;:\\"/\[\]\?=\{\}\s\t]])+))((\s+)*;(\s+)*(((([\p{IsBasicLatin}-[\p{Cc}\(\)<>@,;:\\"/\[\]\?=\{\}\s\t]])+))=((([\p{IsBasicLatin}-[\p{Cc}\(\)<>@,;:\\"/\[\]\?=\{\}\s\t]])+)\|("(([\p{IsLatin-1Supplement}\p{IsBasicLatin}-[\p{Cc}"\n\r]]\|(\s+))\|(\\[\p{IsBasicLatin}]))*"))))*)`

## `ct:ST_Extension`
Restriction: `base=string pattern=([!$&'\(\)\*\+,:=]\|(%[0-9a-fA-F][0-9a-fA-F])\|[:@]\|[a-zA-Z0-9\-_~])+`

## `rel:<Relationships>` : `CT_Relationships`

## `rel:<Relationship>` : `CT_Relationship`

## `rel:CT_Relationships`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `Relationship` | `Relationship` | 0..unbounded |  |  |

## `rel:CT_Relationship`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `string` | `string` |  |  |  |
| attribute | `@TargetMode` | `ST_TargetMode` | optional |  |  |
| attribute | `@Target` | `anyURI` | required |  |  |
| attribute | `@Type` | `anyURI` | required |  |  |
| attribute | `@Id` | `ID` | required |  |  |

## `rel:ST_TargetMode`
Restriction: `base=string`
- `External`
- `Internal`

## `core:<coreProperties>` : `CT_CoreProperties`

## `core:CT_CoreProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `category` | `string` | 0..1 |  |  |
| element | `contentStatus` | `string` | 0..1 |  |  |
| element | `dcterms:created` | `created` | 0..1 |  |  |
| element | `dc:creator` | `creator` | 0..1 |  |  |
| element | `dc:description` | `description` | 0..1 |  |  |
| element | `dc:identifier` | `identifier` | 0..1 |  |  |
| element | `keywords` | `CT_Keywords` | 0..1 |  |  |
| element | `dc:language` | `language` | 0..1 |  |  |
| element | `lastModifiedBy` | `string` | 0..1 |  |  |
| element | `lastPrinted` | `dateTime` | 0..1 |  |  |
| element | `dcterms:modified` | `modified` | 0..1 |  |  |
| element | `revision` | `string` | 0..1 |  |  |
| element | `dc:subject` | `subject` | 0..1 |  |  |
| element | `dc:title` | `title` | 0..1 |  |  |
| element | `version` | `string` | 0..1 |  |  |

## `core:CT_Keywords`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `value` | `CT_Keyword` | 0..unbounded |  |  |
| attribute | `@xml:lang` | `lang` | optional |  |  |

## `core:CT_Keyword`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `string` | `string` |  |  |  |
| attribute | `@xml:lang` | `lang` | optional |  |  |
