using Microsoft.Win32;
using System;

namespace dwag;

public record ThemeColors(Color Background, Color Hover, Color Text);

public static class Theme
{
	public static readonly ThemeColors Light = new(Color.White, Color.LightGray, Color.Black);
	public static readonly ThemeColors Dark = new(Color.Black, Color.DimGray, Color.White);

	private static readonly Lazy<ThemeColors> _currentTheme = new(() =>
	{
		try
		{
			using RegistryKey? key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
			var value = key?.GetValue("AppsUseLightTheme");
			return (value is int intValue && intValue == 1) ? Light : Dark;
		}
		catch
		{
			return Light;
		}
	});

	public static ThemeColors CurrentTheme => _currentTheme.Value;
	public static Color Background => CurrentTheme.Background;
	public static Color Hover => CurrentTheme.Hover;
	public static Color Text => CurrentTheme.Text;
}
