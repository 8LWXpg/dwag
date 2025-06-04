using System.Runtime.InteropServices;

namespace dragger;

public static class FolderIcon
{
	private const uint SHGFI_ICON = 0x100;
	private const uint SHGFI_LARGEICON = 0x0;
	private const uint SHGFI_SMALLICON = 0x1;
	private const uint SHGFI_USEFILEATTRIBUTES = 0x10;
	private const uint FILE_ATTRIBUTE_DIRECTORY = 0x10;

	[StructLayout(LayoutKind.Sequential)]
	private struct SHFILEINFO
	{
		public IntPtr hIcon;
		public IntPtr iIcon;
		public uint dwAttributes;
		[MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)]
		public string szDisplayName;
		[MarshalAs(UnmanagedType.ByValTStr, SizeConst = 80)]
		public string szTypeName;
	};

	[DllImport("shell32.dll", CharSet = CharSet.Unicode)]
	private static extern IntPtr SHGetFileInfo(string pszPath, uint dwFileAttributes,
		ref SHFILEINFO psfi, uint cbSizeFileInfo, uint uFlags);

	[DllImport("user32.dll")]
	private static extern bool DestroyIcon(IntPtr handle);

	/// <summary>
	/// Extracts the icon for a specific folder
	/// </summary>
	/// <param name="folderPath">Path to the folder</param>
	/// <param name="largeIcon">True for large icon, false for small</param>
	/// <returns>Icon object or null if extraction fails</returns>
	public static Icon? ExtractFolderIcon(string folderPath, bool largeIcon = true)
	{
		if (!Directory.Exists(folderPath))
		{
			return null;
		}

		folderPath = Path.GetFullPath(folderPath);

		var shfi = new SHFILEINFO();
		var flags = SHGFI_ICON | (largeIcon ? SHGFI_LARGEICON : SHGFI_SMALLICON);

		var result = SHGetFileInfo(folderPath, FILE_ATTRIBUTE_DIRECTORY,
			ref shfi, (uint)Marshal.SizeOf(shfi), flags);

		if (result == IntPtr.Zero || shfi.hIcon == IntPtr.Zero)
		{
			return null;
		}

		try
		{
			// Clone the icon so we can safely destroy the original handle
			var extractedIcon = Icon.FromHandle(shfi.hIcon);
			var clonedIcon = (Icon)extractedIcon.Clone();
			return clonedIcon;
		}
		finally
		{
			// Always destroy the icon handle to prevent memory leaks
			_ = DestroyIcon(shfi.hIcon);
		}
	}
}
