using System.Reflection;
using System.Text;

namespace dwag;

[AttributeUsage(AttributeTargets.Property, AllowMultiple = false)]
public class ArgAttribute(string? shortName = null, string? description = null) : Attribute
{
	public string? ShortName { get; set; } = shortName;
	public string? Description { get; set; } = description;
}

public class ArgParser
{
	[Arg("h", "Show help")]
	public bool Help { get; private set; }
	[Arg("m", "Move files instead of copy")]
	public bool Move { get; private set; }
	public string[] Files;

	public ArgParser(string[] args)
	{
		Files = [.. args.Where(a => !a.StartsWith('-'))];
		foreach (var flag in args.Where(a => a.StartsWith('-')))
		{
			var flagName = NormalizeFlagName(flag);
			_ = TrySetBooleanProperty(flagName, true);
		}
	}

	private static string NormalizeFlagName(string flag) => flag.TrimStart('-').ToLowerInvariant();

	private bool TrySetBooleanProperty(string flag, bool value)
	{
		PropertyInfo? property = FindPropertyByFlag(flag);

		if (property != null && property.PropertyType == typeof(bool) && property.CanWrite)
		{
			property.SetValue(this, value);
			return true;
		}

		return false;
	}

	private PropertyInfo? FindPropertyByFlag(string flag)
	{
		return GetType()
			.GetProperties()
			.FirstOrDefault(p =>
			{
				ArgAttribute? attr = p.GetCustomAttribute<ArgAttribute>();
				var longName = p.Name.ToLowerInvariant();
				var shortName = attr?.ShortName?.ToLowerInvariant();

				return longName == flag || shortName == flag;
			});
	}

	public string GetHelp()
	{
		StringBuilder sb = new($"{AppDomain.CurrentDomain.FriendlyName} {Application.ProductVersion}\nUsage: {AppDomain.CurrentDomain.FriendlyName} [options] [path]...\nOptions:\n");

		foreach (PropertyInfo? prop in GetType()
			.GetProperties()
			.Where(p => p.Name != nameof(Files))
			.OrderBy(p => p.Name))
		{
			ArgAttribute? attr = prop.GetCustomAttribute<ArgAttribute>();
			var longName = $"--{prop.Name.ToLowerInvariant()}";
			var shortName = attr?.ShortName != null ? $"-{attr.ShortName}" : "";
			var type = prop.PropertyType == typeof(bool) ? "" : $" <{prop.PropertyType.Name.ToLowerInvariant()}>";
			var description = attr?.Description ?? "";

			var flagDisplay = shortName != "" ? $"{shortName}, {longName}" : longName;
			_ = sb.AppendLine($"\t{flagDisplay}{type,-15}\t{description}");
		}

		return sb.ToString();
	}
}
