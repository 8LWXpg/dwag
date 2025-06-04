namespace dragger;

public class DragItem : UserControl
{
	private PictureBox pictureBox;
	private Label label;
	private TableLayoutPanel tablePanel;

	public DragItem(string filePath)
	{
		Dock = DockStyle.Top;
		Height = 40;
		BackColor = Color.Transparent;

		tablePanel = new TableLayoutPanel
		{
			Dock = DockStyle.Fill,
			ColumnCount = 2,
			RowCount = 1,
			Padding = new Padding(10, 0, 0, 0)
		};

		_ = tablePanel.ColumnStyles.Add(new ColumnStyle(SizeType.AutoSize)); // Picture column
		_ = tablePanel.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100F)); // Label column

		pictureBox = new PictureBox
		{
			Size = new Size(24, 24),
			SizeMode = PictureBoxSizeMode.StretchImage,
			Anchor = AnchorStyles.None,
		};

		label = new Label
		{
			AutoSize = true,
			Font = new Font("Segoe UI", 10, FontStyle.Regular),
			Anchor = AnchorStyles.Left,
			TextAlign = ContentAlignment.MiddleLeft,
		};

		tablePanel.Controls.Add(pictureBox, 0, 0);
		tablePanel.Controls.Add(label, 1, 0);

		if (File.Exists(filePath))
		{
			pictureBox.Image = Icon.ExtractAssociatedIcon(filePath)?.ToBitmap();
			label.Text = Path.GetFileName(filePath);
		}
		else
		{
			label.Text = "File not found!";
		}

		Controls.Add(tablePanel);

		tablePanel.MouseEnter += (s, e) => OnMouseEnter(e);
		tablePanel.MouseLeave += (s, e) => OnMouseLeave(e);
		tablePanel.MouseMove += (s, e) => OnMouseMove(e);
		pictureBox.MouseEnter += (s, e) => OnMouseEnter(e);
		pictureBox.MouseLeave += (s, e) => OnMouseLeave(e);
		pictureBox.MouseMove += (s, e) => OnMouseMove(e);
		label.MouseEnter += (s, e) => OnMouseEnter(e);
		label.MouseLeave += (s, e) => OnMouseLeave(e);
		label.MouseMove += (s, e) => OnMouseMove(e);
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
			Height + 10
		);
	}
}
