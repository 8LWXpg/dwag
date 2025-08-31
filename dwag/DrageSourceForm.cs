using Windows.Win32;
using Windows.Win32.Graphics.Dwm;

namespace dwag;

public class DragSourceForm : Form
{
	private readonly string[] _path;

	public DragSourceForm(string[] path)
	{
		unsafe
		{
			var pvAttribute = 1;
			_ = PInvoke.DwmSetWindowAttribute(new(Handle), DWMWINDOWATTRIBUTE.DWMWA_USE_IMMERSIVE_DARK_MODE, &pvAttribute, sizeof(int));
		}

		MouseEnter += (_, _) => BackColor = Theme.Hover;
		MouseLeave += (_, _) => BackColor = Theme.Background;
		MouseMove += DragSource_MouseMove!;

		Padding = new(10, 10, 10, 10);
		BackColor = Theme.Background;
		Cursor = Cursors.Hand;
		TopMost = true;
		Text = AppDomain.CurrentDomain.FriendlyName;
		StartPosition = FormStartPosition.Manual;
		Location = Cursor.Position;

		_path = [.. path
			.Reverse()
			.Select(p => Path.Combine(Directory.GetCurrentDirectory(), p))
			.Where(p => File.Exists(p) || Directory.Exists(p))];

		if (_path.Length == 0)
		{
			_ = MessageBox.Show("Files/folders does not exist", AppDomain.CurrentDomain.FriendlyName);
			Dispose();
		}

		CreateAndSizeForm();
	}

	private void CreateAndSizeForm()
	{
		var maxWidth = 0;
		var totalHeight = 0;

		foreach (var p in _path)
		{
			var item = new DragItem(p);

			// Measure size
			Size itemSize = item.GetSize();
			maxWidth = Math.Max(maxWidth, itemSize.Width);
			totalHeight += itemSize.Height;

			// Add to form and wire events
			Controls.Add(item);
			item.MouseEnter += (_, e) => OnMouseEnter(e);
			item.MouseLeave += (_, e) => OnMouseLeave(e);
			item.MouseMove += (_, e) => OnMouseMove(e);
		}

		// Calculate and set form size
		var formWidth = maxWidth + Padding.Left + Padding.Right;
		var formHeight = totalHeight + Padding.Top + Padding.Bottom + SystemInformation.CaptionHeight + 20;
		Size = new Size(formWidth, formHeight);

		// Prevent resizing
		FormBorderStyle = FormBorderStyle.FixedDialog;
		MaximizeBox = false;
		MinimizeBox = false;
	}

	private void DragSource_MouseMove(object _, MouseEventArgs e)
	{
		if (e.Button != MouseButtons.Left)
		{
			return;
		}

		var dataObject = new DataObject(DataFormats.FileDrop, _path);
		DragDropEffects result = DoDragDrop(dataObject, Globals.ArgParser.Move ? DragDropEffects.Move : DragDropEffects.Copy);
		if (result is DragDropEffects.Move or DragDropEffects.Copy)
		{
			Application.Exit();
		}
	}
}
