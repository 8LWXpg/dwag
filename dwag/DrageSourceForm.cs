namespace dwag;

public class DragSourceForm : Form
{
	private readonly string[] _path;

	public DragSourceForm(string[] path)
	{
		Padding = new(10, 10, 10, 10);
		MouseEnter += (s, e) => BackColor = Color.LightGray;
		MouseLeave += (s, e) => BackColor = Color.White;
		MouseMove += DragSource_MouseMove;
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
			item.MouseEnter += (s, e) => OnMouseEnter(e);
			item.MouseLeave += (s, e) => OnMouseLeave(e);
			item.MouseMove += (s, e) => OnMouseMove(e);
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

	private void DragSource_MouseMove(object sender, MouseEventArgs e)
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
