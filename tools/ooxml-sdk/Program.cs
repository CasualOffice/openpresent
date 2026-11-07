// F-006: validate presentation packages with Microsoft's Open XML SDK.
//
// Usage: Validator <format-version> <deck>...
// Writes one JSON object per deck to stdout:
//   {"deck": "...", "status": "pass"|"fail"|"refused", "reason": "...",
//    "errors": [{"id": "...", "part": "...", "path": "...", "type": "...", "description": "..."}]}
//
// Decks are untrusted. validate.py bounds the archive before this runs, and the SDK is
// opened read-only, with no auto-save, a per-part character cap, and markup
// compatibility processed as the target version would. Descriptions quote attribute
// values from the file, so quoted values are redacted: diagnostics carry names, not content.

using System.Text.Json;
using System.Text.RegularExpressions;
using DocumentFormat.OpenXml;
using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Validation;

static class Program
{
    // Characters per part, matching OoxmlLimits::max_xml_part_bytes as an upper bound.
    const long MaxCharactersInPart = 256L << 20;
    // Errors reported per deck. The count stays exact.
    const int MaxErrors = 1000;

    static readonly Regex Quoted = new("'[^']*'", RegexOptions.Compiled);

    static int Main(string[] args)
    {
        if (args.Length < 2 || !Enum.TryParse<FileFormatVersions>(args[0], out var version))
        {
            Console.Error.WriteLine("usage: Validator <Office2007|...|Microsoft365> <deck>...");
            return 2;
        }
        var failed = false;
        foreach (var path in args.Skip(1))
        {
            var result = ValidateDeck(path, version);
            failed |= (string)result["status"]! != "pass";
            Console.WriteLine(JsonSerializer.Serialize(result));
        }
        return failed ? 1 : 0;
    }

    // Phrases after which the SDK quotes a value from the document.
    static readonly Regex QuotedValue = new(
        @"(invalid value|The string|The text|The value|value of|actual length|length of the value) '[^']*'",
        RegexOptions.Compiled | RegexOptions.IgnoreCase);
    // A qualified or plain XML name, as the SDK quotes element and attribute names.
    static readonly Regex XmlName = new(
        @"^<?([a-z][a-z0-9+.-]*:[^\s'<>]*:)?[A-Za-z_][\w.-]*(:[A-Za-z_][\w.-]*)?>?$", RegexOptions.Compiled);

    static string Redact(string? text)
    {
        if (string.IsNullOrEmpty(text)) return "";
        // Values first, by the phrase that introduces them, whatever they look like:
        // 'dist' is a value there, though it would pass for a name.
        var redacted = QuotedValue.Replace(text, m => m.Value[..m.Value.IndexOf('\'')] + "'...'");
        // Anything else quoted survives only if it is a name.
        return Quoted.Replace(redacted, m =>
        {
            var inner = m.Value.Trim('\'');
            return inner == "..." || (inner.Length <= 256 && XmlName.IsMatch(inner)) ? m.Value : "'...'";
        });
    }

    static Dictionary<string, object?> ValidateDeck(string path, FileFormatVersions version)
    {
        var result = new Dictionary<string, object?>
        {
            ["deck"] = Path.GetFileName(path),
            ["status"] = "pass",
            ["reason"] = null,
        };
        var errors = new List<Dictionary<string, string>>();
        var total = 0;
        try
        {
            var settings = new OpenSettings
            {
                AutoSave = false,
                MaxCharactersInPart = MaxCharactersInPart,
                MarkupCompatibilityProcessSettings = new MarkupCompatibilityProcessSettings(
                    MarkupCompatibilityProcessMode.ProcessAllParts, version),
            };
            using var stream = File.OpenRead(path);
            using var doc = PresentationDocument.Open(stream, false, settings);
            var validator = new OpenXmlValidator(version) { MaxNumberOfErrors = 0 };
            foreach (var e in validator.Validate(doc))
            {
                total++;
                if (errors.Count >= MaxErrors) continue;
                errors.Add(new Dictionary<string, string>
                {
                    ["id"] = e.Id ?? "",
                    ["part"] = e.Part?.Uri.ToString() ?? "",
                    ["path"] = e.Path?.XPath ?? "",
                    ["type"] = e.ErrorType.ToString(),
                    ["description"] = Redact(e.Description),
                });
            }
        }
        catch (Exception ex) when (ex is OpenXmlPackageException or InvalidDataException
                                       or System.Xml.XmlException or IOException
                                       or InvalidOperationException or NotSupportedException
                                       or FileFormatException or ArgumentException
                                       or NamespaceNotUnderstandException)
        {
            // The type, never the message: a message can carry text from the file.
            result["status"] = "refused";
            result["reason"] = ex.GetType().Name;
        }
        if (total > 0) result["status"] = "fail";
        result["errors"] = errors;
        result["total"] = total;
        return result;
    }
}
