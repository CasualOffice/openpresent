# DSIG: Digital signatures

Generated. Tier 1A; render -; edit 10; level Retain. 14 rows.


## `dsig:<SignatureTime>` : `CT_SignatureTime`

## `dsig:<RelationshipReference>` : `CT_RelationshipReference`

## `dsig:<RelationshipsGroupReference>` : `CT_RelationshipsGroupReference`

## `dsig:CT_SignatureTime`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `Format` | `ST_Format` |  |  |  |
| element | `Value` | `ST_Value` |  |  |  |

## `dsig:CT_RelationshipReference`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `string` | `string` |  |  |  |
| attribute | `@SourceId` | `string` | required |  |  |

## `dsig:CT_RelationshipsGroupReference`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `string` | `string` |  |  |  |
| attribute | `@SourceType` | `anyURI` | required |  |  |

## `dsig:ST_Format`
Restriction: `base=string pattern=(YYYY)\|(YYYY-MM)\|(YYYY-MM-DD)\|(YYYY-MM-DDThh:mmTZD)\|(YYYY-MM-DDThh:mm:ssTZD)\|(YYYY-MM-DDThh:mm:ss.sTZD)`

## `dsig:ST_Value`
Restriction: `base=string pattern=(([0-9][0-9][0-9][0-9]))\|(([0-9][0-9][0-9][0-9])-((0[1-9])\|(1(0\|1\|2))))\|(([0-9][0-9][0-9][0-9])-((0[1-9])\|(1(0\|1\|2)))-((0[1-9])\|(1[0-9])\|(2[0-9])\|(3(0\|1))))\|(([0-9][0-9][0-9][0-9])-((0[1-9])\|(1(0\|1\|2)))-((0[1-9])\|(1[0-9])\|(2[0-9])\|(3(0\|1)))T((0[0-9])\|(1[0-9])\|(2(0\|1\|2\|3))):((0[0-9])\|(1[0-9])\|(2[0-9])\|(3[0-9])\|(4[0-9])\|(5[0-9]))(((\+\|-)((0[0-9])\|(1[0-9])\|(2(0\|1\|2\|3))):((0[0-9])\|(1[0-9])\|(2[0-9])\|(3[0-9])\|(4[0-9])\|(5[0-9])))\|Z))\|(([0-9][0-9][0-9][0-9])-((0[1-9])\|(1(0\|1\|2)))-((0[1-9])\|(1[0-9])\|(2[0-9])\|(3(0\|1)))T((0[0-9])\|(1[0-9])\|(2(0\|1\|2\|3))):((0[0-9])\|(1[0-9])\|(2[0-9])\|(3[0-9])\|(4[0-9])\|(5[0-9])):((0[0-9])\|(1[0-9])\|(2[0-9])\|(3[0-9])\|(4[0-9])\|(5[0-9]))(((\+\|-)((0[0-9])\|(1[0-9])\|(2(0\|1\|2\|3))):((0[0-9])\|(1[0-9])\|(2[0-9])\|(3[0-9])\|(4[0-9])\|(5[0-9])))\|Z))\|(([0-9][0-9][0-9][0-9])-((0[1-9])\|(1(0\|1\|2)))-((0[1-9])\|(1[0-9])\|(2[0-9])\|(3(0\|1)))T((0[0-9])\|(1[0-9])\|(2(0\|1\|2\|3))):((0[0-9])\|(1[0-9])\|(2[0-9])\|(3[0-9])\|(4[0-9])\|(5[0-9])):(((0[0-9])\|(1[0-9])\|(2[0-9])\|(3[0-9])\|(4[0-9])\|(5[0-9]))\.[0-9])(((\+\|-)((0[0-9])\|(1[0-9])\|(2(0\|1\|2\|3))):((0[0-9])\|(1[0-9])\|(2[0-9])\|(3[0-9])\|(4[0-9])\|(5[0-9])))\|Z))`
