namespace dragger;

static class Program
{
	/// <summary>
	///  The main entry point for the application.
	/// </summary>
	[STAThread]
	static void Main(string[] args)
	{
		Application.EnableVisualStyles();
		Application.SetCompatibleTextRenderingDefault(true);
		_ = Application.SetHighDpiMode(HighDpiMode.PerMonitorV2);
		Application.EnableVisualStyles();

		if (args.Length == 0)
		{
			_ = MessageBox.Show($"Usage: {AppDomain.CurrentDomain.FriendlyName} [filePath]...");
			return;
		}

		Application.Run(new DragSourceForm(args));
	}
}
