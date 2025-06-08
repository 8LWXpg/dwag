using System.Runtime.InteropServices;
using Windows.Win32;
using Windows.Win32.Storage.FileSystem;
using Windows.Win32.UI.Shell;

namespace dwag;

public static class FolderIcon
{
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

		unsafe
		{
			var shfi = new SHFILEINFOW();
			SHGFI_FLAGS flags = SHGFI_FLAGS.SHGFI_ICON |
				(largeIcon ? SHGFI_FLAGS.SHGFI_LARGEICON : SHGFI_FLAGS.SHGFI_SMALLICON);

			var result = PInvoke.SHGetFileInfo(
				folderPath,
				FILE_FLAGS_AND_ATTRIBUTES.FILE_ATTRIBUTE_DIRECTORY,
				&shfi,
				(uint)Marshal.SizeOf<SHFILEINFOW>(),
				flags);

			if (result == 0 || shfi.hIcon.IsNull)
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
				_ = PInvoke.DestroyIcon(shfi.hIcon);
			}
		}
	}
}
