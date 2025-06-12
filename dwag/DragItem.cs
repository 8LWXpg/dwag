namespace dwag;

public class DragItem : UserControl
{
	private readonly PictureBox pictureBox;
	private readonly Label label;
	private readonly TableLayoutPanel tablePanel;

	public DragItem(string path)
	{
		Dock = DockStyle.Top;
		Height = 40;
		BackColor = Color.Transparent;

		tablePanel = new TableLayoutPanel
		{
			BackColor = Color.Transparent,
			Dock = DockStyle.Fill,
			ColumnCount = 2,
			RowCount = 1,
			Padding = new Padding(10, 0, 0, 0)
		};

		_ = tablePanel.ColumnStyles.Add(new ColumnStyle(SizeType.AutoSize)); // Picture column
		_ = tablePanel.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100F)); // Label column

		pictureBox = new PictureBox
		{
			BackColor = Color.Transparent,
			Size = new Size(24, 24),
			SizeMode = PictureBoxSizeMode.StretchImage,
			Anchor = AnchorStyles.None,
		};

		label = new Label
		{
			BackColor = Color.Transparent,
			ForeColor = Theme.Text,
			AutoSize = true,
			Font = new Font("Segoe UI", 10, FontStyle.Regular),
			Anchor = AnchorStyles.Left,
			TextAlign = ContentAlignment.MiddleLeft,
		};

		tablePanel.Controls.Add(pictureBox, 0, 0);
		tablePanel.Controls.Add(label, 1, 0);

		if (File.Exists(path))
		{
			pictureBox.Image = Icon.ExtractAssociatedIcon(path)?.ToBitmap();
			label.Text = Path.GetFileName(path);
		}
		else if (Directory.Exists(path))
		{
			pictureBox.Image = FolderIcon.ExtractFolderIcon(path)?.ToBitmap();
			label.Text = new DirectoryInfo(path).Name;
		}
		else
		{
			throw new ArgumentException("filePath is not a file or folder");
		}

		Controls.Add(tablePanel);

		tablePanel.MouseEnter += (_, e) => OnMouseEnter(e);
		tablePanel.MouseLeave += (_, e) => OnMouseLeave(e);
		tablePanel.MouseMove += (_, e) => OnMouseMove(e);
		pictureBox.MouseEnter += (_, e) => OnMouseEnter(e);
		pictureBox.MouseLeave += (_, e) => OnMouseLeave(e);
		pictureBox.MouseMove += (_, e) => OnMouseMove(e);
		label.MouseEnter += (_, e) => OnMouseEnter(e);
		label.MouseLeave += (_, e) => OnMouseLeave(e);
		label.MouseMove += (_, e) => OnMouseMove(e);
	}

	public Size GetSize()
	{
		using Graphics g = CreateGraphics();
		SizeF textSize = g.MeasureString(label.Text, label.Font);
		var totalWidth = pictureBox.Width +
						(int)Math.Ceiling(textSize.Width) +
						tablePanel.Padding.Left +
						tablePanel.Padding.Right;
		// Why extra margin?
		return new(
			totalWidth + 45,
			Height
		);
	}
}
